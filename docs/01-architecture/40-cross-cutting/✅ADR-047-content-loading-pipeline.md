---
id: 01-architecture.ADR-047
title: ADR-047 — Content Loading Pipeline
status: approved
owner: architect
created: 2026-06-18
updated: 2026-06-18
supersedes: none
---

# ADR-047: Content 加载管线架构

## 状态

**Approved** — 依赖 ADR-013（Registry & Hot-reload）、ADR-000（Feature Module Map）和 `docs/04-data/infrastructure/registry_schema.md`，本架构决策正式生效。

## 背景

Fre 项目采用数据驱动设计——所有 Definition（AbilityDef、EffectDef、SpellDef 等）存储在 RON 配置文件中，运行时通过 Bevy Asset 系统加载。ADR-013 定义了 Registry 的两层架构（Asset Server + Definition Registry），但缺少具体的加载管线设计：

1. RON 文件如何组织和发现？
2. 19 个 Def 类型如何实现 Asset trait？
3. 加载 → 校验 → 注册的完整流程是什么？
4. 热重载时如何增量更新 Registry 索引？
5. 加载失败如何优雅降级？

## 引用的领域规则与数据架构

- `docs/01-architecture/10-capability-system/ADR-013-registry-hotreload.md` — Registry 两层架构
- `docs/04-data/infrastructure/registry_schema.md` — Registry Schema v2（Handle<T> 存储）
- `docs/04-data/README.md` — Data Law 001（Def-Instance 分离）
- `.trae/rules/架构规则.md` §五 — 数据驱动与内容生产

## 决策

### 1. Def 类型的 Asset 实现

所有 Definition 类型统一派生 `Asset` + `Reflect`，通过 RON 文件加载：

```rust
/// 所有 Definition 的公共基 trait
pub trait DefinitionType: Asset + TypePath {
    const BUCKET_NAME: &'static str;
    const EXTENSION: &'static str = "ron";
}

/// 示例：SpellDef
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize, Reflect)]
#[type_path = "fre::core::domains::spell::components::SpellDef"]
pub struct SpellDef {
    pub id: SpellDefId,
    pub name: String,
    pub level: SpellLevel,
    pub components: SpellComponents,
    pub casting_time: u32,
    pub range: String,
    pub duration: String,
    pub description: String,
}

impl DefinitionType for SpellDef {
    const BUCKET_NAME: &'static str = "spells";
}
```

**关键约束**：
- 所有 Def 字段必须 `Serialize + Deserialize`
- 嵌套类型（如 `SpellComponents`）也必须可序列化
- `#[type_path]` 必须显式指定（Bevy Asset 系统需要）

### 2. RON 文件组织

```
assets/
└── config/
    ├── abilities/          # SpellDef, AbilityDef 等
    │   ├── fireball.ron
    │   └── healing_word.ron
    ├── effects/            # EffectDef
    │   ├── damage_burn.ron
    │   └── heal regeneration.ron
    ├── modifiers/          # ModifierDef
    │   ├── strength_boost.ron
    │   └── poison_weakness.ron
    ├── tags/               # TagHierarchy
    │   ├── element_types.ron
    │   └── damage_types.ron
    ├── attributes/         # AttributeDef
    │   ├── primary_stats.ron
    │   └── derived_stats.ron
    ├── triggers/           # TriggerDef
    │   ├── on_damage_taken.ron
    │   └── on_turn_start.ron
    ├── cues/               # CueDef
    │   ├── fire_impact.ron
    │   └── heal_glow.ron
    ├── items/              # ItemDef
    │   ├── weapons.ron
    │   └── armor.ron
    ├── spells/             # SpellDef
    │   ├── fireball.ron
    │   └── magic_missile.ron
    ├── buffs/              # BuffDef
    │   ├── bless.ron
    │   └── haste.ron
    ├── factions/           # FactionDef
    │   ├── player_faction.ron
    │   └── enemy_faction.ron
    ├── terrains/           # TerrainDef
    │   ├── grassland.ron
    │   └── dungeon_floor.ron
    ├── recipes/            # RecipeDef
    │   ├── iron_sword.ron
    │   └── health_potion.ron
    ├── loot_tables/        # LootTableDef
    │   ├── goblin_drop.ron
    │   └── boss_treasure.ron
    └── quests/             # QuestDef
        ├── main_quest_01.ron
        └── side_quest_01.ron
```

**文件命名约定**：
- 小写 + 下划线（snake_case）
- 扩展名 `.ron`
- 每个文件包含一个 Definition 实例
- 同一类型的多个实例可放在同一目录下

### 3. 加载管线流程

```
┌──────────────────────────────────────────────────────────────────┐
│                    Content Loading Pipeline                       │
│                                                                  │
│  Phase 1: Discovery                                              │
│  ──────────────────                                              │
│  • 扫描 assets/config/ 下所有 .ron 文件                           │
│  • 按目录名映射到对应的 Bucket（abilities/ → abilities bucket）    │
│  • 输出: Vec<(PathBuf, BucketName)>                              │
│                                                                  │
│  Phase 2: Loading                                                │
│  ────────────────                                                │
│  • 对每个 .ron 文件调用 AssetServer::load()                       │
│  • Bevy 内置 RonAssetPlugin 自动反序列化为 Typed Reflect          │
│  • 输出: Vec<Handle<T>>                                          │
│                                                                  │
│  Phase 3: Validation                                             │
│  ──────────────────                                              │
│  • 加载完成后触发 Observer: OnAdd<T>                              │
│  • 执行 DefinitionType::validate() 检查:                         │
│    - ID 格式合法性                                               │
│    - 必填字段完整性                                               │
│    - 引用完整性（依赖的其他 Def 是否存在）                         │
│    - 数值范围合理性                                               │
│  • 输出: Vec<ValidationError>                                    │
│                                                                  │
│  Phase 4: Registration                                           │
│  ────────────────────                                            │
│  • 将 Handle<T> 注册到 RegistryBucket<T>                         │
│  • 更新索引（tag/category 查询）                                  │
│  • 冲突检测（重复 ID 报错）                                       │
│  • 输出: Registry 更新完成                                       │
│                                                                  │
│  Phase 5: Notification                                           │
│  ──────────────────                                              │
│  • 触发 OnDefinitionReloaded 事件                                │
│  • 下游 Observer 响应（如 UI 刷新）                               │
└──────────────────────────────────────────────────────────────────┘
```

### 4. ContentPlugin 实现

```rust
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册所有 Asset 类型
        app.init_asset::<SpellDef>()
          .init_asset::<EffectDef>()
          .init_asset::<ModifierDef>()
          .init_asset::<TagHierarchy>()
          .init_asset::<AttributeDef>()
          .init_asset::<TriggerDef>()
          .init_asset::<CueDef>()
          .init_asset::<ItemDef>()
          .init_asset::<BuffDef>()
          .init_asset::<FactionDef>()
          .init_asset::<TerrainDef>()
          .init_asset::<RecipeDef>()
          .init_asset::<LootTableDef>()
          .init_asset::<QuestDef>()
          .init_asset::<SummonTemplateDef>()
          .init_asset::<ShopDef>()
          .init_asset::<CampEventDef>()
          .init_asset::<DialogueNodeDef>()
          .init_asset::<TargetingDef>();

        // 2. 注册热重载监听
        app.add_observer(on_spell_def_added)
          .add_observer(on_effect_def_added)
          .add_observer(on_modifier_def_added)
          // ... 每种 Def 类型一个 Observer

        // 3. 启动时加载所有配置
        app.add_systems(Startup, load_all_content);
    }
}
```

### 5. 热重载机制

利用 Bevy 原生的 `file_watcher` feature（已在 Cargo.toml 启用）：

```rust
/// 热重载 Observer — 当 Asset 被重新加载时触发
fn on_spell_def_added(
    trigger: Trigger<OnAdd, SpellDef>,
    spells: Res<Assets<SpellDef>>,
    mut registry: ResMut<DefinitionRegistry>,
    mut reload_events: EventWriter<OnDefinitionReloaded>,
) {
    let entity = trigger.target();
    if let Some(spell) = spells.get(entity) {
        // 更新 Registry 索引
        registry.spells.insert(
            DefinitionId::new(spell.id.as_str()),
            Handle::weak(entity),
        );
        // 通知下游
        reload_events.send(OnDefinitionReloaded {
            bucket: "spells".to_string(),
            id: spell.id.as_str().to_string(),
        });
    }
}
```

**热重载安全保证**：
- 已创建的 EffectInstance 使用快照值，不受重载影响
- 新创建的 Effect 使用最新 Definition 值
- Registry 版本号递增，下游可检测变更

### 6. 错误处理策略

| 错误场景 | 处理方式 | 用户感知 |
|----------|----------|----------|
| RON 文件语法错误 | 跳过该文件，日志警告 | 控制台黄色警告 |
| ID 格式非法 | 跳过该 Definition，日志警告 | 控制台黄色警告 |
| 必填字段缺失 | 跳过该 Definition，日志警告 | 控制台黄色警告 |
| 引用不存在的 Def | 标记为 broken reference，继续加载 | 控制台黄色警告 |
| 重复 ID | 报错，后加载的覆盖先加载的 | 控制台红色错误 |
| 文件不存在 | 静默跳过（目录可为空） | 无感知 |

**核心原则**：单个文件失败不应阻止其他文件加载，不应导致游戏崩溃。

### 7. Module Design

```
src/content/
├── mod.rs                    # 模块声明
├── content_plugin.rs         # ContentPlugin（Phase 9 注册）
├── loading/
│   ├── mod.rs
│   ├── discovery.rs          # RON 文件发现与目录映射
│   ├── loader.rs             # AssetServer 加载封装
│   └── validator.rs          # Definition 校验逻辑
├── hot_reload/
│   ├── mod.rs
│   └── observers.rs          # OnAdd<T> Observers
└── tests/
    ├── mod.rs
    ├── unit/
    │   ├── discovery_test.rs
    │   └── validator_test.rs
    └── integration/
        └── load_and_register_test.rs
```

### 8. v1 → v2 Registry 迁移

当前 v1 Registry 使用 `RegistryEntry`（serde_json::Value）直接存储。迁移策略：

| 阶段 | 内容 | 风险 |
|------|------|------|
| Phase 1 | ContentPlugin 加载 RON → Asset，同时写入 v1 Registry（兼容） | 低 |
| Phase 2 | 下游系统逐步从 v1 迁移到 v2（通过 Handle<T> 查询） | 中 |
| Phase 3 | 移除 v1 RegistryEntry 相关代码 | 低 |

**Phase 1 的兼容策略**：ContentPlugin 加载完成后，同时：
1. 将 Handle<T> 注册到 v2 RegistryBucket<T>
2. 将 serde_json::Value 序列化后写入 v1 RegistryEntry（供未迁移的下游使用）

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| RON 文件加载 | AssetServer::load() | file_system → asset_server |
| 加载完成通知 | Observer (OnAdd<T>) | asset_server → content |
| 校验结果 | 同步返回 Vec<ValidationError> | content → content |
| 注册到 Registry | ResMut<DefinitionRegistry> | content → registry |
| 热重载通知 | EventWriter<OnDefinitionReloaded> | content → all features |
| Definition 查询 | Res<DefinitionRegistry> + Res<Assets<T>> | any feature → registry |

## 边界定义

### 允许
- ContentPlugin 读取 assets/config/ 目录下的所有 .ron 文件
- ContentPlugin 注册 Asset 类型到 Bevy Asset 系统
- ContentPlugin 更新 RegistryBucket 索引
- 热重载时增量更新（不影响已运行的 Effect）

### 🟥 禁止
- ContentPlugin 直接修改 Definition 数据（Asset 不可变）
- 单个文件加载失败阻止其他文件加载
- 运行时动态添加新的 Asset 类型（必须在 Plugin::build 中注册）
- ContentPlugin 依赖上层 Feature（只依赖 Infra 层）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 运行时修改 Definition 数据 | 违反 Data Law 001 |
| 加载失败时 panic | 必须优雅降级 |
| 跳过校验直接注册 | 可能导致运行时 panic（引用不存在的 Def） |
| 在非 Startup 阶段加载初始内容 | 时序不可控 |

## 后果

### 正面
- 所有 Definition 统一通过 RON 文件管理，设计师可直接编辑
- 热重载支持，开发体验好
- 校验在加载时完成，运行时不会有配置错误
- 与 Bevy 原生 Asset 系统一致，社区工具可直接使用

### 负面
- 每个新 Def 类型需要：(1) 添加 `#[derive(Asset)]` (2) 在 ContentPlugin 中 `init_asset::<T>()` (3) 创建 Observer
- v1→v2 迁移需要分阶段进行，短期内存在两套查询接口
- RON 文件加载是异步的，Startup 阶段需要等待加载完成

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 编译时嵌入配置（include_str!） | 失去热重载能力，每次修改需重新编译 |
| 运行时 JSON 解析（不走 Asset） | 失去 Bevy 的热重载、依赖追踪、异步加载 |
| 单一大文件存储所有 Definition | 并发编辑冲突大，版本控制困难 |
| 数据库存储 Definition | 过重，不适合游戏配置 |

## 评审要点

- [ ] v1→v2 迁移的兼容策略是否足够平滑？
- [ ] 19 个 Def 类型的 `init_asset` 注册是否可以宏自动化？
- [ ] 热重载时正在执行的 Ability 是否保证使用旧快照？
- [ ] 校验失败的 Definition 是否应该阻止游戏启动？
