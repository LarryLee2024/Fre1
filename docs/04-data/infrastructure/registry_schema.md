---
id: infrastructure.registry.schema.v2
title: Registry Schema — 注册中心数据架构
status: stable
owner: architect
created: 2026-06-16
updated: 2026-06-17
layer: definition
replay-safe: true
supersedes: v1 (direct-value storage)
---

# Registry Schema — 注册中心数据架构

> **领域归属**: Infrastructure | **架构依据**: `ADR-013-registry-hotreload.md`
> **变更说明 v2**: 从直接值存储迁移到 Handle 间接存储，对齐 ADR-013 的两层架构设计

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `DefinitionRegistry` | Definition | 所有 Definition 的全局注册中心，通过 `Handle<T>` 间接持有 Asset |
| `RegistryBucket<T>` | Definition | 类型安全的存储桶，封装 `DefinitionId → Handle<T>` 映射 |
| `IdAllocator` | Definition | ID 分配器（类型前缀 + 数字编号） |
| `RegistryValidation` | Definition | 注册时的一致性校验 |

---

## 2. 架构概览：两层架构

与 ADR-013 一致，Registry 采用两层架构：

```
┌──────────────────────────────────────────────────────────────┐
│  Layer 1: Asset Server (Bevy 原生)                           │
│  ─────────────────────────────────────────────               │
│  • 管理 .ron 文件的加载/热重载                                │
│  • 提供 Asset<T> 句柄和 Assets<T> 资源                        │
│  • Hot-reload 通过 file_watcher feature 实现                  │
│  • 每个 Definition 类型是一个独立 Asset 类型                   │
│  • 注册方式: app.init_asset::<AbilityDef>()                  │
└──────────────────────────────────────────────────────────────┘
                              │ loads / hot-reloads
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  Layer 2: Definition Registry (自定义)                       │
│  ─────────────────────────────────────────────               │
│  • 包装 Asset Server 的查询 API                              │
│  • 提供类型安全的 ID → Handle<T> → &T 查找                    │
│  • 加载时冲突检测（重复 ID）                                  │
│  • 热重载时增量更新索引                                      │
│  • 纯查询接口（不修改 Definition）                             │
└──────────────────────────────────────────────────────────────┘
```

### 2.1 读取路径

```
Consumer System
    │
    ├── registry.abilities.get(id, &assets)
    │       │
    │       ├── 1. id: AbilityDefId → Handle<AbilityDef>
    │       ├── 2. Handle → assets.get(handle) → &AbilityDef
    │       └── 3. 返回 &AbilityDef
    │
    └── registry.abilities.handle(id)
            │
            └── 直接返回 Handle<AbilityDef>（用于延迟访问）
```

---

## 3. Schema Design

### 3.1 DefinitionRegistry（Definition 层）

```rust
/// 全局 Definition 注册中心。
/// 存储 Asset Handle，不直接持有数据。
/// Layer 7 (Infrastructure) Resource
#[derive(Resource)]
pub struct DefinitionRegistry {
    /// 按类型分桶存储的 Handle 映射
    pub abilities: RegistryBucket<AbilityDef>,
    pub effects: RegistryBucket<EffectDef>,
    pub modifiers: RegistryBucket<ModifierDef>,
    pub tags: RegistryBucket<TagHierarchy>,
    pub attributes: RegistryBucket<AttributeDef>,
    pub triggers: RegistryBucket<TriggerDef>,
    pub cues: RegistryBucket<CueDef>,
    pub items: RegistryBucket<ItemDef>,
    pub spells: RegistryBucket<SpellDef>,
    pub buffs: RegistryBucket<BuffDef>,
    pub factions: RegistryBucket<FactionDef>,
    pub terrains: RegistryBucket<TerrainDef>,
    pub recipes: RegistryBucket<RecipeDef>,
    pub loot_tables: RegistryBucket<LootTableDef>,
    pub quests: RegistryBucket<QuestDef>,

    /// 自定义 Def 扩展（Domain 可注册自定义 Def 类型）
    pub custom: HashMap<String, RegistryBucket<BoxedDef>>,
}
```

### 3.2 RegistryBucket<T>（Definition 层）

```rust
/// 每个 Definition 类型的 Handle 存储桶。
/// 只存储 Handle<T>，数据由 Bevy Assets<T> 持有。
pub struct RegistryBucket<T: Asset> {
    /// DefinitionId → Handle<T> 映射（加载后不可变）
    items: HashMap<DefinitionId, Handle<T>>,
    /// 按 tag/category 建立的索引
    indices: HashMap<IndexKey, Vec<DefinitionId>>,
    /// 变更追踪（用于热重载通知）
    version: u64,
}

impl<T: Asset> RegistryBucket<T> {
    /// 通过 DefinitionId 查询，返回数据引用
    pub fn get<'a>(
        &self,
        id: &DefinitionId,
        assets: &'a Assets<T>,
    ) -> Option<&'a T> {
        self.items.get(id).and_then(|h| assets.get(h))
    }

    /// 直接返回 Handle（用于 AssetServer 的延迟访问）
    pub fn handle(&self, id: &DefinitionId) -> Option<Handle<T>> {
        self.items.get(id).cloned()
    }

    /// 遍历所有已注册的 Def
    pub fn iter(&self) -> impl Iterator<Item = (&DefinitionId, &Handle<T>)> {
        self.items.iter()
    }

    /// 注册一个新的 Def Handle
    pub(crate) fn insert(&mut self, id: DefinitionId, handle: Handle<T>) {
        self.items.insert(id, handle);
        self.version += 1;
    }

    /// 按索引查询
    pub fn query_index(&self, key: &IndexKey) -> Vec<DefinitionId> {
        self.indices.get(key).cloned().unwrap_or_default()
    }
}
```

### 3.3 DefinitionType Trait

```rust
/// 每个 Definition 类型实现此 Trait。
/// 使 Asset 加载系统与 Registry 配合工作。
pub trait DefinitionType: Asset + TypePath {
    type Config: DeserializeOwned;
    const BUCKET_NAME: &'static str;
    const EXTENSION: &'static str;

    /// 从 RON 配置创建 Asset
    fn from_config(config: Self::Config) -> Result<Self, ConfigError>;

    /// 加载后校验（ID 格式、引用完整性）
    fn validate(&self, registry: &DefinitionRegistry) -> Result<(), ValidationError>;
}
```

### 3.4 IdAllocator（Definition 层）

```rust
/// ID 分配器：管理各类型前缀的数字编号分配。
pub struct IdAllocator {
    allocators: HashMap<IdType, AllocatorState>,
}

struct AllocatorState {
    /// 类型前缀
    prefix: &'static str,
    /// 当前最大已分配编号
    next_id: u64,
    /// 已释放/回收的 ID（可选）
    recycled: Vec<u64>,
    /// ID 格式（总位数，0-padded）
    digit_count: u8,
}

pub enum IdType {
    Ability,
    Effect,
    Trigger,
    Tag,
    Attribute,
    Cue,
    Item,
    Spell,
    Quest,
    Faction,
    Terrain,
    Recipe,
    Buff,
    LootTable,
    Custom(String),
}
```

### 3.5 RegistryValidation（Definition 层）

```rust
/// 注册时的一致性校验结果。
pub struct RegistryValidation {
    pub has_errors: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    /// 跨 Def 引用检查（所有引用的 ID 都存在）
    pub cross_references: CrossReferenceReport,
}

pub struct CrossReferenceReport {
    pub total_defs: u32,
    pub total_references: u32,
    pub broken_references: Vec<BrokenReference>,
}

pub struct BrokenReference {
    pub source_def: String,
    pub field: String,
    pub referenced_id: String,
    pub expected_type: String,
}
```

### 3.6 事件

```rust
/// 热重载完成时触发的事件
#[derive(Event)]
pub struct OnDefinitionReloaded {
    pub bucket_name: &'static str,
    pub new_version: u64,
    pub changed_ids: Vec<DefinitionId>,
}
```

### 3.7 Snapshot（运行时一致性保证）

```rust
/// 快照 — 在 Ability 激活时创建，保证已触发的 Effect 不受热重载影响
pub struct AbilitySnapshot {
    pub ability_def_id: AbilityDefId,
    pub snapshot_version: u64,       // 当时 Registry 版本号
    pub snapshot_time: GameTime,     // 施法游戏内时间
    pub effect_snapshots: Vec<EffectSnapshot>,
}

pub struct EffectSnapshot {
    pub effect_def_id: EffectDefId,
    pub values: HashMap<String, f32>,  // 关键数值快照
    pub rng_seed: u64,
}
```

---

## 4. RegistryPlugin 初始化流程

```rust
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app
            // 1. 初始化空的 Registry
            .init_resource::<DefinitionRegistry>()

            // 2. 注册所有 Def 为 Bevy Asset 类型
            .init_asset::<AbilityDef>()
            .init_asset::<EffectDef>()
            .init_asset::<ModifierDef>()
            .init_asset::<TagHierarchy>()
            .init_asset::<AttributeDef>()
            .init_asset::<TriggerDef>()
            .init_asset::<CueDef>()
            .init_asset::<ItemDef>()
            .init_asset::<SpellDef>()
            .init_asset::<BuffDef>()
            .init_asset::<FactionDef>()
            .init_asset::<TerrainDef>()
            .init_asset::<RecipeDef>()
            .init_asset::<LootTableDef>()
            .init_asset::<QuestDef>()

            // 3. 注册热重载监听
            .add_systems(PostUpdate, (
                on_asset_modified::<AbilityDef>,
                on_asset_modified::<EffectDef>,
                // ... 所有 Def 类型
            ))

            // 4. 初始化时加载所有配置文件（由 ContentPlugin 驱动）
            .add_systems(Startup, init_registry);
    }
}
```

> **注**: 实际文件加载逻辑属于 ContentPlugin（横切层），RegistryPlugin 只提供注册能力的 Resource。

---

## 5. 热重载策略

### 5.1 流程

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
on_asset_modified::<T> (Observer)
       │
       ├── Registry 更新索引和版本号
       └── 触发 OnDefinitionReloaded 事件
              │
              ▼
     下游 Observer 响应重载
     (如 UI 刷新、Spec 重新选择)
```

### 5.2 运行时安全的 Definition 替换

| 场景 | 行为 | 风险 |
|------|------|------|
| 修改数值（如伤害 10→15） | 新创建的 Effect 使用新值，已生效的 Effect 继续使用旧 snapshot | ✅ 安全 |
| 新增 Definition | 加入 Registry，已加载内容不受影响 | ✅ 安全 |
| 删除 Definition | Registry 标记为 deprecated，引用此 ID 的 Spec 触发警告 | ⚠️ 存档兼容检查 |
| 修改 ID | 视为删除旧 + 新增新 | ⚠️ 需要迁移 |

---

## 6. Layer Analysis

| 层 | 组件 | 说明 |
|----|------|------|
| Definition | `RegistryBucket<T>.items` | Handle 指向的 Asset 数据 |
| Spec | `AbilitySnapshot`, `EffectSnapshot` | 运行时快照（保护 Effect 不受热重载影响） |
| Instance | 无 | Registry 不参与运行时实例状态 |
| Persistence | 无 | Registry 不在存档中持久化 |

---

## 7. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → 全部 Def Schema | 持有所有 Def 类型的 Handle 映射 |
| 依赖 | → `foundation/id_strategy.md` | ID 分配策略 |
| 被依赖 | ← 所有 Capabilities Schema | 所有系统在加载/运行时查询 Registry |
| 被依赖 | ← 所有 Business Domain Schema | 业务域通过 Registry 获取 Def 数据 |

---

## 8. Validation Rules

| # | 规则 | 触发时机 |
|---|------|----------|
| V1 | ID 格式正确（前缀 + 6位数字） | Def 注册 |
| V2 | ID 全局唯一（跨类型也唯一） | Def 注册 |
| V3 | 所有跨 Def 引用有效 | 加载完成时全量校验 |
| V4 | Deprecated 的 Def 不再被引用 | 加载完成时全量校验 |
| V5 | 循环依赖检测 | 加载完成时全量校验 |
| V6 | Asset Handle 可解析 | 注册时（handle 指向的 Asset 已加载） |

---

## 9. Replay / Save Compatibility

Registry 是内容加载阶段的基础设施，不参与运行时回放。存档不包含 Registry 数据（存档只存 Instance 和 Persistence 层数据）。

**热重载不影响回放确定性**：回放使用快照值（`EffectSnapshot`），不受热重载影响。

---

## 10. Constitution Check

| 条款 | 合规 | 说明 |
|------|------|------|
| Data Driven First | ✅ | Registry 由配置驱动，代码不硬编码 Def |
| Single Source of Truth | ✅ | 所有 Def 通过 Registry 查询，禁止重复定义 |
| Def-Instance 强制分离 | ✅ | Registry 只存 Handle，数据在 Assets<T>（Def 层） |
| Rule-Content 强制分离 | ✅ | Registry 只做查询，不含业务逻辑 |
| Replay 优先于便利 | ✅ | 热重载通过 Snapshot 保护，不破坏回放确定性 |

---

## 附：v1 → v2 变更记录

| 变更项 | v1 (直接值) | v2 (Handle 间接) | 原因 |
|--------|------------|-----------------|------|
| 存储方式 | `HashMap<Id, T>` | `HashMap<Id, Handle<T>>` | 原生支持热重载 |
| 查询方式 | `registry.abilities.get(id)` → `&T` | `registry.abilities.get(id, &assets)` → `Option<&T>` | 需传入 Assets<T> Resource |
| Asset 注册 | 未定义 | `app.init_asset::<T>()` | 对齐 Bevy 原生 Asset 系统 |
| 热重载 | 需手动实现 | Bevy Asset 原生支持 | 减少自定义代码 |
| Snapshot | 未定义 | `AbilitySnapshot`, `EffectSnapshot` | 保护运行时 Effect |
| 事件 | 未定义 | `OnDefinitionReloaded` | 热重载通知 |
