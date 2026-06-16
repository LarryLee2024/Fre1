---
id: 01-architecture.ADR-001
title: ADR-001 — Plugin Composition & Registration Order
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-001: Plugin 组合与注册顺序

## 状态

**Proposed** — 等待 @data-architect 确认 Schema 加载顺序需求。

## 背景

Bevy 中 Plugin 是模块化的核心单元。30+ 个领域模块需要以正确的顺序注册，确保：
- Asset（Def）在下游 Systems 执行前加载完成
- Resource 在依赖它的 Feature 注册前已初始化
- State 在依赖状态转换的 Systems 注册前就绪
- Schedule 和 SystemSet 的 before/after 关系不会循环

## 引用的领域规则与数据架构

- `docs/04-data/README.md` — 四层数据架构、Asset 生命周期
- `.trae/rules/ECS规则.md` — Schedule 权责划分、Plugin 设计规范
- `.trae/rules/架构规则.md` — Plugin 是唯一对外入口

## 决策

### 1. Plugin 注册采用严格的分层顺序

```
App::new()
    // ════════════════════════════════════════════
    // Phase 0: Core Bevy + Shared (L0)
    // ════════════════════════════════════════════
    .add_plugins(DefaultPlugins)
    .add_plugins(shared::SharedPlugin)

    // ════════════════════════════════════════════
    // Phase 1: Capabilities — Foundation (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::capabilities::tag::TagPlugin)
    .add_plugins(core::capabilities::attribute::AttributePlugin)
    .add_plugins(core::capabilities::modifier::ModifierPlugin)
    .add_plugins(core::capabilities::aggregator::AggregatorPlugin)
    .add_plugins(core::capabilities::gameplay_context::GameplayContextPlugin)

    // ════════════════════════════════════════════
    // Phase 2: Capabilities — Logic Skeleton (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::capabilities::spec::SpecPlugin)
    .add_plugins(core::capabilities::condition::ConditionPlugin)
    .add_plugins(core::capabilities::trigger::TriggerPlugin)
    .add_plugins(core::capabilities::event::EventPlugin)

    // ════════════════════════════════════════════
    // Phase 3: Capabilities — Behavior (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::capabilities::ability::AbilityPlugin)
    .add_plugins(core::capabilities::targeting::TargetingPlugin)
    .add_plugins(core::capabilities::execution::ExecutionPlugin)
    .add_plugins(core::capabilities::effect::EffectPlugin)
    .add_plugins(core::capabilities::stacking::StackingPlugin)
    .add_plugins(core::capabilities::cue::CuePlugin)

    // ════════════════════════════════════════════
    // Phase 4: Capabilities — Runtime (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::capabilities::runtime::RuntimePlugin)

    // ════════════════════════════════════════════
    // Phase 5: Business Domains — Foundation (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::domains::tactical::TacticalPlugin)
    .add_plugins(core::domains::terrain::TerrainPlugin)
    .add_plugins(core::domains::faction::FactionPlugin)

    // ════════════════════════════════════════════
    // Phase 6: Business Domains — Core (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::domains::combat::CombatPlugin)
    .add_plugins(core::domains::spell::SpellPlugin)
    .add_plugins(core::domains::reaction::ReactionPlugin)
    .add_plugins(core::domains::progression::ProgressionPlugin)
    .add_plugins(core::domains::inventory::InventoryPlugin)
    .add_plugins(core::domains::party::PartyPlugin)
    .add_plugins(core::domains::camp_rest::CampRestPlugin)

    // ════════════════════════════════════════════
    // Phase 7: Business Domains — Narrative & Economy (L1 Core)
    // ════════════════════════════════════════════
    .add_plugins(core::domains::narrative::NarrativePlugin)
    .add_plugins(core::domains::quest::QuestPlugin)
    .add_plugins(core::domains::economy::EconomyPlugin)
    .add_plugins(core::domains::crafting::CraftingPlugin)
    .add_plugins(core::domains::summon::SummonPlugin)

    // ════════════════════════════════════════════
    // Phase 8: Infrastructure (L2)
    // ════════════════════════════════════════════
    .add_plugins(infra::registry::RegistryPlugin)
    .add_plugins(infra::pipeline::PipelinePlugin)
    .add_plugins(infra::replay::ReplayPlugin)
    .add_plugins(infra::save::SavePlugin)
    .add_plugins(infra::input::InputPlugin)

    // ════════════════════════════════════════════
    // Phase 9: Cross-cutting
    // ════════════════════════════════════════════
    .add_plugins(app::AppPlugin)
    .add_plugins(content::ContentPlugin)
    #[cfg(feature = "dev")]
    .add_plugins(tools::DevToolsPlugin)
    .add_plugins(modding::ModdingPlugin)
```

### 2. Plugin 内部结构规范

每个 Feature 的 Plugin 统一结构：

```rust
// plugin.rs — Feature 的唯⼀对外入口
pub struct MyFeaturePlugin;

impl Plugin for MyFeaturePlugin {
    fn build(&self, app: &mut App) {
        app
            // 1. Resources（如适用）
            .init_resource::<MyFeatureResource>()

            // 2. Events
            .add_event::<MyFeatureEvent>()

            // 3. Entities（如 World 初始化时需要）
            // .add_systems(Startup, spawn_initial_entities)

            // 4. Systems（按 Schedule 分组）
            .add_systems(PreUpdate, (
                system_a,
                system_b,
            ))
            .add_systems(Update, (
                system_c,
                system_d,
            ))
            .add_systems(PostUpdate, (
                system_e,
            ))

            // 5. Observers
            .observe(on_my_feature_event)

            // 6. States（如适用）
            // .init_state::<MyFeatureState>()
            ;
    }
}
```

### 3. Plugin 间依赖声明

当 Plugin B 需要确保 Plugin A 已注册时，使用 `Plugin::build` 中的依赖检查：

```rust
// 方式一：在 Plugin build 中检查必要 Resource
pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        if app.world().get_resource::<EffectRegistry>().is_none() {
            panic!("CombatPlugin requires EffectPlugin to be registered first");
        }
        // ...
    }
}
```

```rust
// 方式二：将 Phase 包装为 PluginGroup
pub struct GamePlugins;
impl PluginGroup for GamePlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Phase 1: Capabilities Foundation
            .add(core::capabilities::tag::TagPlugin)
            .add(core::capabilities::attribute::AttributePlugin)
            // Phase 2: Business Domains
            .add(core::domains::tactical::TacticalPlugin)
            .add(core::domains::combat::CombatPlugin)
            // ...
    }
}
```

> 🟩 **推荐方式二**（PluginGroup），方式一用于调试期防御。

### 4. 条件编译与 Feature 开关

```rust
// Phase 分组支持 feature flags
.add_plugins((
    // 核心插件（永远加载）
    CorePlugins,
    // 开发工具（仅 dev feature）
    #[cfg(feature = "dev")]
    dev_tools::DevToolsPlugin,
    // 测试工具（仅 test build）
    #[cfg(test)]
    test_helpers::TestPlugin,
))
```

## Module Design

所有 Plugin 的注册集中在 `src/lib.rs` 中，按 Phase 分组排列，每个 Phase 上方有明确的注释分隔线。当 Phase 内 Plugin 过多时，拆入 `src/plugin_groups.rs`。

## Communication Design

Plugin 本身不通信。Plugin 负责注册 Systems/Events/Resources，通信由 Systems 在运行时通过 Events 进行。

## 边界定义

### 允许
- Plugin 在 `build()` 中检查依赖 Resource 是否存在（防御式）
- 使用 `PluginGroup` 包装多个 Plugin 为一个组
- 条件编译控制 Plugin 注册

### 🟥 禁止
- Plugin 在 `build()` 中修改另一个 Plugin 注册的 Resource
- Plugin 在 `build()` 中执行任何运行时逻辑（仅注册）
- 跳过 Plugin 直接访问另一个 Feature 的内部模块
- Plugin 依赖关系隐式（必须显式从低层到高层注册）

## Forbidden

- 🟥 **禁止循环 Plugin 依赖**：A 在 build 中 require B，B 在 build 中 require A
- 🟥 **禁止 Plugin 全局状态**：Plugin 结构体本身不存储状态，状态放在 Resource 中
- 🟥 **禁止 Plugin 耦合**：Plugin A 不得直接创建 Plugin B 所需的 Entity/Component
- 🟥 **禁止运行时动态注册**：Plugin 必须在 App 构建期全部注册完毕，禁止运行时 `add_plugins`

## Definition / Instance Design

Plugin 是编译时静态结构，不直接涉及运行时数据。本 ADR 不产生新的 Definition/Instance。

## 后果

### 正面
- 30+ 个 Plugin 按阶段注册，依赖关系线性清晰
- PluginGroup 机制让上层可以一次注册整个游戏
- 条件编译天然支持 dev/test 环境
- Plugin 内部结构统一，代码审查可预期

### 负面
- 注册顺序硬编码在 `lib.rs` 中，新 Plugin 需要找到正确 Phase 插入
- Phase 数量多（10 个），但这是领域数量多的必然结果

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 隐式依赖（Plugin 自行 `add_plugins` 依赖） | 一个 Plugin 可以触发整个依赖树，注册顺序不透明 |
| 单一巨型 Plugin | 违反单一职责，无法独立测试 |
| Bevy DefaultPlugins 风格全展开 | 35 个 Plugin 在调用点全展开，可读性差 |

## 评审要点

- [ ] 是否缺少必要的前置 Plugin？
- [ ] Capabilities Foundation → Logic Skeleton → Behavior → Runtime 的顺序是否合理？
- [ ] `GamePluginGroup` 是否应该拆分为子 Group（CapabilityGroup / DomainGroup / InfraGroup）？
- [ ] Infrastructure (L2) 在 Domains 之后注册是否正确（Domains 需先注册 rules，Infra 随后提供管线支持）？
