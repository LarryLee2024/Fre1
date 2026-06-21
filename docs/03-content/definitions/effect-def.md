---
id: 03-content.definitions.effect-def
title: EffectDef — Effect Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# EffectDef — Effect Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/effect_domain.md` | **数据 Schema**: `docs/04-data/capabilities/effect_schema.md` | **插件代码**: `src/content/plugins/effect_plugin.rs`

---

## 1. Overview

EffectDef 是整个能力系统所有"结果"的载体定义——伤害、治疗、Buff、Debuff、地形变化、召唤物，一切最终都表现为 Effect。EffectDef 定义了一个效果**做什么**（Modifier 修改、Tag 变更）、**持续多久**（Instant/Duration/Infinite）、**按什么节奏**（周期 Tick）、**如何堆叠**（Stacking 策略）、**如何表现**（Cue 绑定）。

### 关键设计原则

- **EffectDef ≠ AbilityDef**：EffectDef 只定义效果本身，不定义谁触发、何时触发、如何选择目标——这些由 AbilityDef 编排
- **EffectDef 是业务执行的唯一入口**：所有对属性/标签的修改必须通过 Effect 系统，禁止外部系统直接修改属性
- **EffectDef 可独立注册**：EffectDef 是独立可引用的 Asset，可在多个 AbilityDef 和 BuffDef 之间共享

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `effect_domain.md` | Effect 的四阶段生命周期、来源追溯规则、与 Stacking/Modifier 的交互 |
| `effect_schema.md` | EffectDef 的完整字段定义、EffectDuration、EffectPeriod、StackingConfig 的数据结构 |
| `modifier-def.md` | 本 Def 的 `modifiers` 字段引用的 ModifierConfig 和 ModifierDefId |
| `condition-def.md` | 本 Def 的 `application_condition` 字段类型 |
| `cue-def.md` | 本 Def 的 `cues` 字段引用的 CueDefId |
| `execution-def.md` | 本 Def 的 `execution` 字段类型 |
| `stacking-def.md` | 本 Def 的 `stacking` 字段引用的 StackingDefId |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Effect 定义——描述一个效果的所有静态属性。
///
/// EffectDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<EffectDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct EffectDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: EffectId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── 持续时间 ──
    /// 持续时间类型：瞬时/有限回合/无限
    pub duration: EffectDuration,

    /// 周期 Tick 参数（仅 Duration 类型有效，可选）
    pub period: Option<EffectPeriod>,

    // ── 属性与标签修改 ──
    /// 效果携带的修改器列表（应用时注册到目标属性）
    pub modifiers: Vec<ModifierConfig>,

    /// 可选的 ModifierDef 引用列表（与 modifiers 合并后应用）
    pub modifier_defs: Option<Vec<ModifierDefId>>,

    /// 效果授予的标签（应用时添加到目标实体）
    pub granted_tags: Vec<TagId>,

    /// 效果需要的标签（目标必须拥有效果才能生效）
    pub required_tags: Option<Vec<TagId>>,

    /// 目标不能拥有的标签（否则效果应用失败，用于免疫检查）
    ///
    /// 来源：Data Schema `docs/04-data/capabilities/effect_schema.md` §3.1
    pub ignored_tags: Option<Vec<TagId>>,

    /// 效果移除时清理的标签
    pub removed_tags: Option<Vec<TagId>>,

    /// 应用此效果时，移除目标上具有这些标签的其他效果
    ///
    /// 来源：Data Schema `docs/04-data/capabilities/effect_schema.md` §3.1
    pub remove_effects_with_tags: Option<Vec<TagId>>,

    // ── 条件与执行 ──
    /// 应用条件（可选，满足此条件效果才能应用）
    pub application_condition: Option<ConditionDefId>,

    /// 关联的执行计算（可选，Instant 类效果需要）
    pub execution: Option<ExecutionConfig>,

    /// 可选的 ExecutionDef 引用
    pub execution_def: Option<ExecutionDefId>,

    // ── 堆叠策略 ──
    /// 效果叠加策略（可选，默认不堆叠）
    pub stacking: Option<StackingConfig>,

    /// 可选的 StackingDef 引用（与 stacking 互斥，优先使用 inline config）
    pub stacking_def: Option<StackingDefId>,

    // ── 表现 ──
    /// 视觉/听觉表现信号绑定
    pub cues: Vec<CueBinding>,

    /// 效果图标 Key（可选）
    pub icon_key: Option<String>,

    // ── 元数据 ──
    /// 效果分类
    pub effect_category: EffectCategory,
    /// 是否可见（在 UI 中显示）
    pub visible: bool,
    /// 是否可被驱散
    pub dispellable: bool,
    /// 显示优先级（UI 排序用）
    pub display_priority: u8,
}
```

### 内嵌数据结构

以下是 EffectDef 引用的关键内嵌数据结构。完整定义见对应的数据 Schema 文档。

```rust
/// 持续时间类型
#[derive(Deserialize, Clone, Debug)]
pub enum EffectDuration {
    Instant,
    HasDuration { turns: u32, frames: Option<u64>, calculation: DurationCalculation },
    Infinite,
}

/// 周期 Tick 定义
#[derive(Deserialize, Clone, Debug)]
pub struct EffectPeriod {
    pub interval_turns: u32,
    pub interval_frames: Option<u64>,
    pub initial_delay: Option<u64>,
    pub max_ticks: Option<u32>,
    pub tick_execution: Option<ExecutionConfig>,
}

/// 修改器配置（嵌入在 EffectDef 中使用）
#[derive(Deserialize, Clone, Debug)]
pub struct ModifierConfig {
    pub op: ModifierOp,
    pub target_attribute: AttributeId,
    pub value: ScalableValue,
    pub priority: u8,
    pub source: ModifierSource,
    pub modifier_def_id: Option<ModifierDefId>,
    pub filter: Option<ModifierFilter>,
}

/// 执行计算配置（嵌入在 EffectDef 中使用）
#[derive(Deserialize, Clone, Debug)]
pub struct ExecutionConfig {
    pub execution_type: ExecutionType,
    pub execution_def_id: Option<ExecutionDefId>,
}

/// Cue 绑定
#[derive(Deserialize, Clone, Debug)]
pub struct CueBinding {
    pub cue_tag: CueTag,
    pub cue_def_id: CueDefId,
    pub delay_frames: Option<u64>,
}

/// 堆叠配置（嵌入在 EffectDef 中使用）
#[derive(Deserialize, Clone, Debug)]
pub struct StackingConfig {
    pub stacking_type: StackingType,
    pub max_stacks: u32,
    pub allow_cross_source: bool,
    pub overflow_behavior: OverflowBehavior,
    pub reapply_modifiers_on_stack: bool,
}
```

### 字段说明

- **`schema_version`**: 当前为 1。版本升级时通过 Migration 层自动升级旧 Def 数据
- **`modifier_defs`** vs **`modifiers`**: 两者合并后应用。`modifier_defs` 引用已注册的 ModifierDef，`modifiers` 内联定义。使用 ModifierDef 引用时支持跨 Def 共享和全局修改
- **`execution`** vs **`execution_def`**: 两者互斥。`execution` 内联定义 ExecutionConfig，`execution_def` 引用已注册的 ExecutionDef
- **`stacking`** vs **`stacking_def`**: 两者互斥。`stacking` 内联定义 StackingConfig，`stacking_def` 引用已注册的 StackingDef

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// EffectDef 注册插件
pub struct EffectDefPlugin;

impl Plugin for EffectDefPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册 Asset 类型
        app.register_asset::<EffectDef>();

        // 2. 注册 AssetLoader
        app.init_asset_loader::<RonAssetLoader<EffectDef>>();

        // 3. 创建 DefRegistry 资源
        app.insert_resource(DefRegistry::<EffectDef>::new());

        // 4. 注册加载/校验/注册管线
        app.add_systems(
            PreUpdate,
            load_effect_defs
                .run_if(resource_changed::<Assets<EffectDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 EffectDef
pub fn get_effect_def(effect_id: &EffectId, registry: &DefRegistry<EffectDef>) -> Option<&EffectDef> {
    registry.get(effect_id)
}

/// 按标签过滤 EffectDef
pub fn get_effect_defs_by_tag(tag_id: &TagId, registry: &DefRegistry<EffectDef>) -> Vec<&EffectDef> {
    registry.iter().filter(|def| def.tags_contain(tag_id)).collect()
}
```

### DefRegistry 提供的能力

- `registry.get(id: &EffectId) -> Option<&EffectDef>` — 按 ID 精确查找
- `registry.iter() -> impl Iterator<Item = &EffectDef>` — 遍历所有 Def
- `registry.count() -> usize` — 获取总数
- `registry.contains(id: &EffectId) -> bool` — 判断是否存在
- `registry.dependencies(id: &EffectId) -> Vec<DefDependency>` — 获取依赖关系
- `registry.freeze()` — 冻结注册表（加载完成后调用，禁止后续变更）

### 注册生命周期

```
EffectDefPlugin::build
  │
  ├── EffectDef 从 assets/config/01_capabilities/effects.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │     └── 校验: RON 语法正确性、枚举合法性
  │
  ├── Validate
  │     ├── ID 唯一性检查
  │     ├── 引用存在性检查 (TagId, ConditionDefId, ModifierDefId, ...)
  │     ├── 字段合法性检查 (priority 范围, duration 参数, ...)
  │     └── 依赖图循环检查
  │
  ├── Register (注入 DefRegistry<EffectDef>)
  │
  └── Freeze (管线完成后不可变)
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | EffectId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1，不兼容版本拒绝加载 |
| V3 | `duration` 参数合法 | HasDuration.turns > 0 或 frames > 0，Infinite 无需参数 |
| V4 | `period.interval_turns >= 1` | 周期 Ticks 至少间隔 1 回合 |
| V5 | `period.max_ticks >= 1` (如果设置) | 最大 Tick 次数至少为 1 |
| V6 | `display_priority` 范围 | 0-100，默认 50 |
| V7 | `effect_category` 有效 | 必须匹配 EffectCategory 的已知变体（Custom 除外） |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `modifier_defs` 中的每个 ModifierDefId 已注册 | 在 DefRegistry<ModifierDef> 中存在 |
| V9 | `modifiers[].target_attribute` 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V10 | `modifiers[].modifier_def_id` (如果设置) 已注册 | 在 DefRegistry<ModifierDef> 中存在 |
| V11 | `execution_def` (如果设置) 已注册 | 在 DefRegistry<ExecutionDef> 中存在 |
| V12 | `stacking_def` (如果设置) 已注册 | 在 DefRegistry<StackingDef> 中存在 |
| V13 | `application_condition` (如果设置) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V14 | `cues[].cue_def_id` 已注册 | 在 DefRegistry<CueDef> 中存在 |
| V15 | `granted_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V16 | `required_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V17 | `ignored_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V18 | `removed_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V19 | `remove_effects_with_tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 互斥字段校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | `execution` 和 `execution_def` 不可同时设置 | 两者互斥，只能选其一。Instant 类型可两者皆无（通过 modifiers 直接生效） |
| V19 | `stacking` 和 `stacking_def` 不可同时设置 | inline config 和 Def 引用互斥 |
| V20 | Duration::Instant 类型不能有 `period` | 瞬时效果不能有周期 Tick |
| V21 | Duration::Instant 类型若同时无 `execution` 和 `execution_def`，则必须至少有一个 `modifiers` | 瞬时效果需通过 execution 或 modifiers 产生实际效果 |

### 4.4 依赖图校验

| # | 规则 | 说明 |
|---|------|------|
| V22 | EffectDef 不得引用自身 | `effect_def_id` 不得为自身 ID |
| V23 | EffectDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |
| V24 | EffectDef 依赖图不得形成循环 | Effect → ConditionDef → (其他) → Effect 的路径不允许 |

---

## 5. RON 示例

```ron
(
    // EffectDef 示例：灼烧 DOT（持续火焰伤害）
    id: "eff:burn",
    name_key: "effect.eff_burn.name",
    description_key: "effect.eff_burn.desc",
    schema_version: 1,

    // 持续 3 回合
    duration: HasDuration(
        turns: 3,
        calculation: Fixed,
    ),

    // 每回合 Tick 一次
    period: Some((
        interval_turns: 1,
        tick_execution: Some((
            execution_type: Damage(
                formula_id: "dot_fire_damage",
                damage_type: ["tag:damage_type_fire"],
                damage_dice: Some((count: 1, sides: 6)),
                can_critical: false,
            ),
        )),
        max_ticks: Some(3),
    )),

    // 敏捷 -2 的 Debuff
    modifiers: [
        (
            op: Add,
            target_attribute: "attr:dexterity",
            value: Fixed(-2.0),
            priority: 50,
            source: (
                source_type: Buff,
                source_id: "eff:burn",
            ),
        ),
    ],

    // 燃烧期间目标获得 "burning" 标签
    granted_tags: ["tag:status_burning"],

    // 需要目标没有火焰免疫
    application_condition: Some("cond:has_no_fire_immunity"),

    // 可堆叠，最多 5 层
    stacking: Some((
        stacking_type: Aggregate,
        max_stacks: 5,
        allow_cross_source: true,
        overflow_behavior: IgnoreNew,
        reapply_modifiers_on_stack: false,
    )),

    // 表现绑定
    cues: [
        (cue_tag: OnApply, cue_def_id: "cue:fire_spark", delay_frames: None),
        (cue_tag: OnTick, cue_def_id: "cue:burn_tick", delay_frames: None),
        (cue_tag: OnRemove, cue_def_id: "cue:fire_extinguish", delay_frames: None),
    ],

    effect_category: Debuff,
    visible: true,
    dispellable: true,
    display_priority: 60,
)
```

---

## 6. 与 BuffDef 的关系

EffectDef 和 BuffDef 是互补的 Def 类型：

| 对比维度 | EffectDef | BuffDef |
|----------|-----------|---------|
| 本质 | 效果定义本身 | 持久状态的 Effect 容器 |
| 是否独立存在 | 是（可被 AbilityDef 直接引用） | 是（包装一个 EffectDef） |
| 有无额外元数据 | 基础元数据 | 状态分类、UI 显示规则、免疫规则 |
| 典型使用 | 嵌入在 Ability 效果链中即时执行 | 作为独立 Buff 实体持久管理 |
| 存储位置 | `effects.ron` | `buffs.ron` |
