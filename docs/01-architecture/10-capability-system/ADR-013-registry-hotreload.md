---
id: 01-architecture.ADR-013
title: ADR-013 — Registry & Hot-reload Architecture
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-013: Registry 与热重载架构

## 状态

**Proposed** — 依赖 ADR-000（Feature Module Map）和 `docs/04-data/infrastructure/registry_schema.md`。

## 背景

Fre 项目采用数据驱动设计——所有 Definition（AbilityDef、EffectDef、TagHierarchy 等）存储在 RON 配置文件中，在运行时加载为 Bevy Asset。需要一个统一的 Registry 系统来：

1. 管理所有 Definition 的 ID 注册和冲突检测
2. 支持热重载（开发期配置文件变更无需重启）
3. 提供跨 Feature 的 Definition 查询接口
4. 确保 Definition 运行时不可变（Data Law 001）

## 引用的领域规则与数据架构

- `docs/04-data/infrastructure/registry_schema.md` — Registry Schema
- `docs/04-data/foundation/id_strategy.md` — ID 策略
- `docs/04-data/README.md` — Data Law 001（Def-Instance 分离）
- `.trae/rules/架构规则.md` §五 — 数据驱动与内容生产
- `.trae/rules/编码规则.md` — Definition 不可变

## 决策

### 1. Registry 两层架构

```
┌──────────────────────────────────────────────────────────────┐
│  Layer 1: Asset Server (Bevy 原生)                           │
│  ─────────────────────────────────────────────               │
│  • 管理 .ron 文件的加载/热重载                                │
│  • 提供 Asset<T> 句柄                                        │
│  • Hot-reload 通过 file_watcher feature 实现                  │
│  • 每个 Definition 类型是一个独立 Asset 类型                  │
└──────────────────────────────────────────────────────────────┘
                              │ loads
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  Layer 2: Definition Registry (自定义)                       │
│  ─────────────────────────────────────────────               │
│  • 包装 Asset Server 的查询 API                              │
│  • 提供类型安全的 ID → &Definition 查找                       │
│  • 加载时冲突检测（重复 ID）                                  │
│  • 热重载时增量更新（不影响已运行的 Effect）                    │
│  • 纯查询接口（不修改 Definition）                             │
└──────────────────────────────────────────────────────────────┘
```

### 2. Registry Resource 设计

```rust
/// 全局 Registry — 所有 Definition 的查询入口
/// Layer 7 (Infrastructure) Resource
#[derive(Resource)]
pub struct DefinitionRegistry {
    /// 按类型分桶存储的 Definition 数据
    abilities: RegistryBucket<AbilityDef>,
    effects: RegistryBucket<EffectDef>,
    modifiers: RegistryBucket<ModifierDef>,
    tags: RegistryBucket<TagHierarchy>,
    attributes: RegistryBucket<AttributeDef>,
    triggers: RegistryBucket<TriggerDef>,
    cues: RegistryBucket<CueDef>,
    items: RegistryBucket<ItemDef>,
    spells: RegistryBucket<SpellDef>,
    buffs: RegistryBucket<BuffDef>,
    factions: RegistryBucket<FactionDef>,
    terrains: RegistryBucket<TerrainDef>,
    recipes: RegistryBucket<RecipeDef>,
    loot_tables: RegistryBucket<LootTableDef>,
    quests: RegistryBucket<QuestDef>,
}

/// 每个 Definition 类型的存储桶
pub struct RegistryBucket<T: DefinitionType> {
    /// ID → Definition 映射（加载后不可变）
    items: HashMap<DefinitionId, Handle<T>>,
    /// 索引：方便按 tag/category 查询
    indices: HashMap<IndexKey, Vec<DefinitionId>>,
    /// 变更追踪（用于热重载通知）
    version: u64,
}
```

### 3. Definition 查询与使用

```rust
/// 查询方式一：通过 Registry（推荐）
fn resolve_ability(registry: Res<DefinitionRegistry>, id: AbilityDefId) -> Option<&AbilityDef> {
    registry.abilities.get(id)
}

/// 查询方式二：通过 Handle + Asset Server（当需要 Asset 内部可变性时）
fn use_ability_asset(
    registry: Res<DefinitionRegistry>,
    asset_server: Res<AssetServer>,
    abilities: Res<Assets<AbilityDef>>,
    id: AbilityDefId,
) {
    let handle = registry.abilities.handle(id)?;
    let ability = abilities.get(handle)?;
    // ...
}
```

### 4. 热重载策略

#### 4.1 热重载流程

```
配置文件修改 (file_watcher 检测)
       │
       ▼
Asset Server 重新加载 .ron
       │
       ▼
Assets<T> 中的 Asset 被替换
       │
       ▼
on_asset_changed<T> (Observer)
       │
       ├── Registry 更新索引
       ├── 版本号 +1
       └── 触发 OnDefinitionReloaded 事件
              │
              ▼
    下游 Observer 响应重载
    (如 Talbe 需要刷新显示的技能描述)
```

#### 4.2 运行时安全的 Definition 替换

| 场景 | 行为 | 风险 |
|------|------|------|
| 修改数值（如伤害 10→15） | 新创建的 Effect 使用新值，已生效的 Effect 继续使用旧 snapshot | ✅ 安全 |
| 新增 Definition | 加入 Registry，已加载内容不受影响 | ✅ 安全 |
| 删除 Definition | Registry 标记为 deprecated，引用此 ID 的 Spec 触发警告 | ⚠️ 存档兼容检查 |
| 修改 ID | 视为删除旧 + 新增新 | ⚠️ 需要迁移 |

#### 4.3 快照机制保证一致性

Spec 层在施法时对 Definition 进行快照，保证已触发的 Effect 不受热重载影响：

```rust
/// 快照 — 在 Ability 激活时创建
pub struct AbilitySnapshot {
    pub ability_def_id: AbilityDefId,
    pub snapshot_version: u64,       // 当时 Registry 版本号
    pub snapshot_time: GameTime,     // 施法游戏内时间
    pub effect_snapshots: Vec<EffectSnapshot>,
}

pub struct EffectSnapshot {
    pub effect_def_id: EffectDefId,
    pub values: HashMap<String, f32>,  // 关键数值快照（攻击力、倍率等）
    pub rng_seed: u64,                 // 施法时的 RNG 种子
}
```

### 5. 配置文件的 Asset 加载

```rust
/// 每个 Definition 类型实现此 Trait
pub trait DefinitionType: Asset + TypePath {
    type Config: DeserializeOwned;
    const BUCKET_NAME: &'static str;
    const EXTENSION: &'static str;

    /// 从 RON 配置创建 Asset
    fn from_config(config: Self::Config) -> Result<Self, ConfigError>;
    /// 加载后校验
    fn validate(&self, registry: &DefinitionRegistry) -> Result<(), ValidationError>;
}
```

### 6. Registry 初始化流程

```rust
pub struct RegistryPlugin;
impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app
            // 1. 初始化空的 Registry
            .init_resource::<DefinitionRegistry>()

            // 2. 注册所有 Asset 类型
            .register_asset_type::<AbilityDef>()
            .register_asset_type::<EffectDef>()
            .register_asset_type::<ModifierDef>()
            // ... 所有 Definition 类型

            // 3. 注册热重载监听
            .add_systems(PostUpdate, (
                on_asset_changed::<AbilityDef>,
                on_asset_changed::<EffectDef>,
                // ...
            ))

            // 4. 初始化时加载所有配置文件
            .add_systems(Startup, load_all_definitions);
    }
}
```

## Module Design

```
src/registry/
  ├── plugin.rs              — RegistryPlugin
  ├── resources.rs           — DefinitionRegistry, RegistryBucket
  ├── systems.rs             — load_all_definitions, on_asset_changed
  ├── events.rs              — OnDefinitionReloaded
  ├── api.rs                 — 公开查询函数
  └── internal/
      ├── loader.rs          — RON 加载与解析
      ├── validator.rs       — 定义校验（ID 冲突、引用完整性）
      └── snapshot.rs        — Snapshot 管理
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| Asset 加载完成 | Observer (`OnAdd<AbilityDef>`) | asset_server → registry |
| 热重载完成 | Message (`OnDefinitionReloaded`) | registry → 所有 Feature |
| Definition 查询 | 直接调用 `registry.abilities.get(id)` | 任何 Feature → registry |
| 冲突检测 | 同步调用（`validate()`） | registry 内部 |
| 快照 | `AbilitySnapshot` struct（非 ECS 通信） | ability → effect |

## 边界定义

### 允许
- 任何 Feature 通过 `Res<DefinitionRegistry>` 读取 Definition
- 热重载时更新 Registry 索引
- Asset 加载时 ID 冲突检测

### 🟥 禁止
- 运行时修改 `RegistryBucket` 中的 Definition 数据（Asset 本身不可变）
- Definition 类型在多个 Feature 中重复注册
- Asset 加载失败导致游戏崩溃（必须优雅降级）
- Registry 持有可变引用到 Asset 内部数据

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 运行时修改 Definition 数据 | 违反 Data Law 001 |
| Spec 层不创建快照 | 热重载后旧 Effect 与当前值不一致 |
| 热重载时销毁正在运行的 Effect | Effect 应继续使用旧 Snapshot |
| ID 冲突静默覆盖 | 必须报错，导致加载失败 |
| Registry 依赖上层 Feature | Registry 是 Layer 7，不能依赖业务层 |

## Definition / Instance Design

Registry 本身不属于 Definition/Instance 分层，它是 Definition 的**管理和查询层**：
- 存储的数据是 Definition（通过 Handle<T> 持有）
- Registry 本身是 Resource（Instance 层）

## 后果

### 正面
- Registry 统一了所有 Definition 的查询入口
- 热重载+快照机制保证开发体验和运行时一致性
- ID 冲突在加载时检测，不推迟到运行时
- Asset 加载流程与 Bevy 原生 Asset 系统一致

### 负面
- 每个新 Definition 类型需要在 Registry 中注册（模板代码）
- 热重载时已创建的 Spec 不会自动刷新（需要用户手动重新选择）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 每个 Feature 自己管理 Asset Handle | 没有统一查询入口，跨 Feature 查询困难 |
| 不使用 Registry，全局 HashMap | 失去类型安全，热重载通知困难 |
| Registry 持有 Arc<RwLock<T>> 可变引用 | 违反 Definition 不可变原则 |
| 直接使用 Bevy Asset 的 `Assets<T>` | 需要包装才能提供 ID 查询 + 冲突检测 |

## 评审要点

- [ ] `RegistryBucket` 是否需要并发访问支持（多线程 System）？
- [ ] 热重载时如何处理正在执行的 Ability？Snapshot 机制是否覆盖所有场景？
- [ ] 是否需要 Registry 的全局调试/Inspector 面板？
- [ ] 新增 Definition 类型时，需要修改多少个文件（模板代码量评估）？
