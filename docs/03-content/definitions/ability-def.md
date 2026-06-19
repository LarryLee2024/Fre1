---
id: 03-content.definitions.ability-def
title: AbilityDef — Ability Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# AbilityDef — Ability Content Def 定义

> **Content Layer**: L1 Capability (组合终端) | **领域规则**: `docs/02-domain/capabilities/ability_domain.md` | **数据 Schema**: `docs/04-data/capabilities/ability_schema.md` | **插件代码**: `src/content/plugins/ability_plugin.rs`

---

## 1. Overview

AbilityDef 是 L1 Capability 层的**组合终端**——它引用所有 9 个 L1 Def 类型（ConditionDef、TriggerDef、TargetingDef、ExecutionDef、EffectDef、ModifierDef、StackingDef、CueDef、TagDef），将它们编排为一个完整的能力（技能/法术/天赋/反应）。

### 核心定位

- **编排者，非执行者**：AbilityDef 定义 Condition 检查顺序、Cost 消耗规则、Targeting 选择方案、Effect 应用链，但不包含这些步骤的逻辑实现——实现交由各 Capability 领域
- **组合终端，非 God Def**：AbilityDef 不直接定义伤害值、持续时间、目标形状——这些通过引用 EffectDef/TargetingDef 等 Def 来获取。AbilityDef 是"引用"的集合，不是"数据"的集合
- **唯一激活入口**：所有技能必须通过 Ability 领域的激活流程进入执行，禁止绕过 AbilityDef 直接触发 Effect

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `ability_domain.md` | 能力生命周期（Ready/Casting/Active/Cooldown/Blocked）、状态转换规则、组合优于创建原则 |
| `ability_schema.md` | AbilityDef 完整字段、AbilityCategory、ActivationType、CostDef、CooldownDef、EffectApplication、LevelScaling、Restrictions 的数据结构 |
| `condition-def.md` | 本 Def 的 `conditions` 字段引用的 ConditionDefId 类型 |
| `trigger-def.md` | 本 Def 的 `triggers` 字段引用的 TriggerDefId 类型 |
| `targeting-def.md` | 本 Def 的 `targeting` 字段使用的 TargetType/Shape/PriorityRule 类型 |
| `effect-def.md` | 本 Def 的 `effect_chain` 字段引用的 EffectDefId 类型 |
| `execution-def.md` | 间接引用（通过 EffectDef） |
| `modifier-def.md` | 间接引用（通过 EffectDef） |
| `stacking-def.md` | 本 Def 的 `stacking` 字段引用的 StackingDefId 类型 |
| `cue-def.md` | 间接引用（通过 EffectDef） |

### 依赖关系全景

```
AbilityDef (组合终端)
  ├──→ ConditionDef   (activation_condition, EffectNode 内联条件)
  ├──→ TriggerDef     (仅 Passive/Reaction 类型)
  ├──→ TargetingDef   (内联 Embedding，不独立注册)
  ├──→ EffectDef      (effect_chain[].effect_def_id)
  │     ├──→ ExecutionDef   (间接)
  │     ├──→ ModifierDef    (间接)
  │     ├──→ StackingDef    (间接)
  │     └──→ CueDef         (间接)
  ├──→ StackingDef    (可选，技能级堆叠规则)
  └──→ TagDef         (tags 字段)

不直接引用 L0+ 层：内容通过 Effect → Modifier 间接引用 L0 AttributeDef
不直接引用 L2+ 层：Entity/Gameplay/World 层内容通过 Event 间接交互
```

### 关键设计原则

- **Effect 是唯一业务执行入口**：AbilityDef 不直接修改属性，所有修改通过 Effect → Modifier 管线
- **Condition 先于 Cost**：激活流程中 Condition 检查必须先于 Cost 消耗（不变性规则）
- **Cost 和 Cooldown 复用 Effect 机制**：不造 CostSystem/CooldownSystem，Cost = Attribute + Effect，Cooldown = Tag + Effect(Duration)
- **AbilityType 决定行为约束**：Passive 类型不能有 Cost/Cooldown，Reaction 类型必须关联 Trigger

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Ability 定义——L1 Capability 组合终端。
///
/// AbilityDef 编排 Condition/Trigger/Targeting/Effect 等所有 L1 Def，
/// 形成一个完整的能力（技能/法术/天赋/反应）。
///
/// 经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<AbilityDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct AbilityDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: AbilityId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── 能力类型 ──
    /// 能力大类：Active/Passive/Reaction/Innate
    pub ability_type: AbilityType,

    /// 最大等级（用于 LevelScaling）
    pub max_level: u8,

    // ── 资源消耗 ──
    /// 资源消耗列表（仅 Active 类型有效，Passive 类型必须为空）
    pub costs: Vec<CostDef>,

    // ── 冷却 ──
    /// 冷却规则（仅 Active/Reaction 类型有效）
    pub cooldown: Option<CooldownDef>,

    // ── 前提条件 ──
    /// 激活条件列表——所有条件必须满足才能激活（AND 语义）
    ///
    /// 引用 ConditionDef。Condition 检查先于 Cost 消耗，见能力领域不变量 3.1。
    pub activation_conditions: Vec<ConditionId>,

    // ── 触发事件（仅 Passive/Reaction 类型） ──
    /// 触发条件列表——监听的事件→技能激活
    ///
    /// Active 类型的 triggers 必须在运行时被忽略。
    pub triggers: Vec<TriggerId>,

    // ── 目标选择 ──
    /// 目标选择规则（内联定义，引用 TargetingDef 中的类型）
    pub targeting: TargetingConfig,

    // ── 效果链 ──
    /// 效果链——技能激活后按顺序执行的效果节点
    ///
    /// 每个节点可包含 EffectDef 引用、参数覆盖、独立条件、
    /// 执行延迟、触发概率、独立 Cue 绑定。
    pub effect_chain: Vec<EffectNodeDef>,

    // ── 技能级堆叠 ──
    /// 技能级堆叠策略（可选，控制技能实例的堆叠行为）
    pub stacking: Option<StackingId>,

    // ── 等级缩放 ──
    /// 技能等级参数（每级的数值变化，可选）
    pub level_scaling: Option<LevelScaling>,

    // ── 限制条件 ──
    /// 可用职业/种族/等级/前置技能限制（可选）
    pub restrictions: Option<Restrictions>,

    // ── 元数据 ──
    /// 能力标签（用于过滤、分类、免疫检查）
    pub tags: Vec<TagId>,

    /// 能力元数据（可见性、可打断性、图标、动画）
    pub metadata: AbilityMetadata,
}

// ═══════════════════════════════════════════
// 内嵌数据结构
// ═══════════════════════════════════════════

/// 能力类型
///
/// 决定技能的激活方式、生命周期约束、UI 显示。
#[derive(Deserialize, Clone, Debug)]
pub enum AbilityType {
    /// 主动技能——需要玩家手动激活
    Active {
        /// 施法方式
        activation: ActivationType,
    },
    /// 被动技能——常驻效果，无需激活，没有 Cost/Cooldown
    Passive,
    /// 反应技能——回合外自动响应触发事件
    Reaction {
        /// 响应方式
        activation: ActivationType,
    },
    /// 内在能力——种族/职业自带，不可移除，不可遗忘
    Innate,
}

/// 激活方式
#[derive(Deserialize, Clone, Debug)]
pub enum ActivationType {
    /// 瞬发（无施法时间，立即生效）
    Instant,
    /// 需要施法时间（帧数）
    CastTime { frames: u64 },
    /// 需要保持专注（被打断则失效）
    Concentration,
    /// 需要蓄力（可中断，蓄力越久效果越强）
    Charge { max_charge_frames: u64 },
}

/// 资源消耗定义
#[derive(Deserialize, Clone, Debug)]
pub struct CostDef {
    /// 消耗的资源属性 ID（如法力值、体力值）
    pub resource_attribute: AttributeId,

    /// 消耗量（正数表示消耗，负数表示获得）
    pub amount: ScalableValue,

    /// 是否可选消耗（如"可消耗最多 3 层法术位"）
    pub optional: bool,

    /// 消耗类型
    pub cost_type: CostType,
}

/// 消耗类型
#[derive(Deserialize, Clone, Debug)]
pub enum CostType {
    /// 固定消耗
    Fixed,
    /// 百分比当前值（如消耗当前 HP 的 20%）
    PercentageCurrent,
    /// 百分比最大值（如消耗最大 MP 的 30%）
    PercentageMax,
    /// 每级额外消耗
    PerLevel { base: f32, per_level: f32 },
}

/// 冷却定义
#[derive(Deserialize, Clone, Debug)]
pub struct CooldownDef {
    /// 冷却时长（回合数）
    pub turns: u32,

    /// 冷却是否从技能激活时开始计时
    /// true: 激活即开始（技能执行期间也在走冷却）
    /// false: 执行完毕后才开始
    pub starts_on_activate: bool,

    /// 是否与其他技能共享冷却
    pub shared_cooldown_group: Option<String>,

    /// 强制冷却 Tag（可选，自定义冷却标签名称）
    pub cooldown_tag: Option<TagId>,
}

/// 目标选择配置
///
/// 内联定义，不引用独立注册的 TargetingDef。
/// 这是因为目标选择是 Ability 的核心流程步骤且通常不会跨技能复用。
#[derive(Deserialize, Clone, Debug)]
pub struct TargetingConfig {
    /// 目标类别（自身/友方/敌方/所有等）
    pub target_type: TargetType,

    /// 范围形状（单体/圆形/直线/锥形/链式/爆炸/墙体）
    pub shape: TargetShape,

    /// 最大射程（网格单位，None = 无限制）
    pub range: Option<f32>,

    /// 最小射程（None = 无限制）
    pub min_range: Option<f32>,

    /// 最大目标数
    pub max_targets: u32,

    /// 是否允许选择施法者自身
    pub include_self: bool,

    /// 排除条件（满足此条件的目标不被选中）
    pub exclude_condition: Option<ConditionId>,

    /// 附加过滤条件（只选中满足此条件的目标）
    pub filter_condition: Option<ConditionId>,

    /// 是否需要视野
    pub require_los: bool,

    /// 是否忽略障碍物
    pub ignore_obstacles: bool,

    /// 能否选择已死亡实体
    pub allow_dead_targets: bool,

    /// 优先级排序规则（多个可选目标时的自动选择）
    pub priority_rule: Option<PriorityRule>,
}

/// 效果链节点——效果链中的一环
///
/// 技能激活后按顺序执行 effect_chain 中的每个 EffectNodeDef。
/// 每个节点可独立控制目标选择、应用条件、执行延迟、触发概率。
#[derive(Deserialize, Clone, Debug)]
pub struct EffectNodeDef {
    /// 引用的 EffectDef ID（必须已注册）
    pub effect_def_id: EffectId,

    /// 效果参数覆盖（可选，覆盖 EffectDef 中的默认值）
    pub override_params: Option<EffectOverride>,

    /// 该效果的目标选择覆盖（可选，默认使用技能的 targeting 结果）
    pub targeting_override: Option<TargetingConfig>,

    /// 该效果的应用条件（可选，在效果链中独立判断）
    ///
    /// 内联引用 ConditionDef。条件不满足时跳过此节点继续执行后续节点。
    pub condition: Option<ConditionId>,

    /// 执行延迟（帧数，效果链中延迟执行本效果）
    pub delay_frames: Option<u64>,

    /// 执行概率（0.0–1.0，用于"有几率触发"的效果）
    pub chance: Option<f32>,

    /// 表现信号绑定（可选，覆盖 EffectDef 中的 cues）
    pub cues: Option<Vec<CueId>>,
}

/// 效果参数覆盖
#[derive(Deserialize, Clone, Debug)]
pub struct EffectOverride {
    pub duration_override: Option<ScalableValue>,
    pub magnitude_override: Option<ScalableValue>,
    pub period_override: Option<u64>,
}

/// 等级缩放
#[derive(Deserialize, Clone, Debug)]
pub struct LevelScaling {
    /// 每级伤害增加值
    pub damage_per_level: Option<ScalableValue>,
    /// 每级范围增加值
    pub range_per_level: Option<f32>,
    /// 每级消耗变化
    pub cost_per_level: Option<Vec<(AttributeId, ScalableValue)>>,
    /// 每级冷却变化
    pub cooldown_reduction_per_level: Option<u32>,
    /// 等级突破阈值（如 3 级/5 级时质变）
    pub breakpoints: Vec<LevelBreakpoint>,
}

/// 等级突破点
#[derive(Deserialize, Clone, Debug)]
pub struct LevelBreakpoint {
    /// 触发等级
    pub level: u8,
    /// 质变描述
    pub description_key: LocalizationKey,
    /// 新增效果（可选）
    pub additional_effects: Vec<EffectId>,
}

/// 限制条件
#[derive(Deserialize, Clone, Debug)]
pub struct Restrictions {
    /// 所需职业标签
    pub required_class: Option<Vec<TagId>>,
    /// 所需种族标签
    pub required_race: Option<Vec<TagId>>,
    /// 最低等级
    pub min_level: Option<u8>,
    /// 所需属性条件
    pub required_attributes: Option<Vec<AttributeCheck>>,
    /// 所需前置技能
    pub prerequisite_abilities: Option<Vec<AbilityId>>,
}

/// 能力元数据
#[derive(Deserialize, Clone, Debug)]
pub struct AbilityMetadata {
    /// 是否在快捷栏中显示
    pub visible: bool,
    /// 是否可被打断
    pub interruptible: bool,
    /// 图标资源 Key
    pub icon_key: Option<String>,
    /// 施法动画资源 Key
    pub cast_animation: Option<String>,
}

// ═══════════════════════════════════════════
// 外部引入的类型（定义在对应 Def 文档中）
// ═══════════════════════════════════════════

// TargetType, TargetShape, PriorityRule — 见 targeting-def.md
// ScalableValue — 见 modifier_schema.md
// AttributeCheck — 见 condition-def.md
// AttributeId — L0 AttributeDef
// TagId — L0 TagDef
// ConditionId — ConditionDef
// TriggerId — TriggerDef
// EffectId — EffectDef
// StackingId — StackingDef
// CueId — CueDef
```

### 字段说明

- **`ability_type`**: 决定技能的行为约束。Active 技能可设置 Cost/Cooldown；Passive 技能不能有 Cost/Cooldown（常驻）；Reaction 技能必须有关联的 Trigger；Innate 技能不可被移除
- **`costs`**: 技能激活的消耗列表。每个 CostDef 引用一个 AttributeDef。多个 cost 使用 AND 语义（所有消耗同时扣除）。对于 Passive 类型，验证规则强制其为空
- **`cooldown`**: 冷却规则。starts_on_activate = true 时，"技能执行期间也在走冷却"（适合长时间技能）；shared_cooldown_group 用于同组技能共享冷却（如所有"火焰技能"共享 1 回合冷却）
- **`activation_conditions`**: 激活条件的列表。所有条件必须满足（AND 语义）。执行顺序在条件检查阶段并行判断（无短路逻辑）
- **`triggers`**: 仅在 Passive/Reaction 类型有效。对于 Active 技能该字段可空，运行时自动忽略
- **`targeting`**: 内联定义而非引用 TargetingDef，因为目标选择通常紧密耦合于具体技能难以复用。但在效果链中可通过 `targeting_override` 让某个效果节点使用不同目标选择
- **`effect_chain`**: 技能的核心负载——效果节点列表。按顺序执行，每个节点可独立控制条件、延迟、概率。效果链长度默认上限为 10（可在配置中调整）
- **`stacking`**: 技能级堆叠策略，控制多个技能实例之间的关系（如"冲锋"技能不能同时激活两次）
- **`level_scaling`**: 等级缩放——支持伤害/范围/消耗/冷却随等级变化，以及等级突破点（新增效果）
- **`restrictions`**: 使用限制——职业、种族、等级、属性、前置技能条件。限制了技能的可学习/可装备条件
- **`metadata`**: UI 显示控制、可打断性、资源 Key 映射

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// AbilityDef 注册插件
pub struct AbilityDefPlugin;

impl Plugin for AbilityDefPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册 Asset 类型
        app.register_asset::<AbilityDef>();

        // 2. 注册 AssetLoader
        app.init_asset_loader::<RonAssetLoader<AbilityDef>>();

        // 3. 创建 DefRegistry 资源
        app.insert_resource(DefRegistry::<AbilityDef>::new());

        // 4. 注册加载/校验/注册管线
        app.add_systems(
            PreUpdate,
            load_ability_defs
                .run_if(resource_changed::<Assets<AbilityDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 AbilityDef
pub fn get_ability_def(id: &AbilityId, registry: &DefRegistry<AbilityDef>) -> Option<&AbilityDef> {
    registry.get(id)
}

/// 按 AbilityType 过滤
pub fn get_abilities_by_type(
    ability_type: AbilityType,
    registry: &DefRegistry<AbilityDef>,
) -> Vec<&AbilityDef> {
    registry.iter()
        .filter(|def| matches_ability_type(&def.ability_type, &ability_type))
        .collect()
}

/// 按 Tag 过滤
pub fn get_abilities_by_tag(tag_id: &TagId, registry: &DefRegistry<AbilityDef>) -> Vec<&AbilityDef> {
    registry.iter()
        .filter(|def| def.tags.contains(tag_id))
        .collect()
}
```

### DefRegistry 提供的能力

- `registry.get(id: &AbilityId) -> Option<&AbilityDef>` — 按 ID 精确查找
- `registry.iter() -> impl Iterator<Item = &AbilityDef>` — 遍历所有 Def
- `registry.count() -> usize` — 获取总数
- `registry.contains(id: &AbilityId) -> bool` — 判断是否存在
- `registry.dependencies(id: &AbilityId) -> Vec<DefDependency>` — 获取依赖关系（跨层引用解析）
- `registry.freeze()` — 冻结注册表（加载完成后调用，禁止后续变更）

### 跨层引用解析

AbilityDef 的引用覆盖 9 个 L1 Def 类型和 2 个 L0 Def 类型。跨层引用解析在注册管线中递归进行：

```
AbilityDef 注册时需解析：
  ├── Tier 1（直接引用）
  │     activation_conditions → DefRegistry<ConditionDef>
  │     triggers              → DefRegistry<TriggerDef>
  │     effect_chain[].effect_def_id → DefRegistry<EffectDef>
  │     stacking              → DefRegistry<StackingDef>
  │     tags                  → DefRegistry<TagDef>
  │
  ├── Tier 2（通过 EffectDef 间接引用）
  │     EffectDef:
  │       ├── application_condition → DefRegistry<ConditionDef>
  │       ├── modifiers[].target_attribute → DefRegistry<AttributeDef>
  │       ├── execution_def → DefRegistry<ExecutionDef>
  │       ├── stacking_def  → DefRegistry<StackingDef>
  │       └── cues[].cue_def_id → DefRegistry<CueDef>
  │
  └── Tier 3（通过 ExecutionDef 间接引用）
        ExecutionDef:
          ├── damage_type → DefRegistry<TagDef>
          └── attribute_modifier.source_attribute → DefRegistry<AttributeDef>
```

### 注册生命周期

```
AbilityDefPlugin::build
  │
  ├── AbilityDef 从 assets/config/01_capabilities/abilities.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │     └── 校验: RON 语法正确性、枚举合法性
  │
  ├── Validate
  │     ├── ID 唯一性检查（与其他 AbilityDef 不重复）
  │     ├── 字段级校验（数值范围、类型约束）
  │     ├── 引用存在性检查（跨所有 9 个 Registry）
  │     ├── AbilityType 约束检查（Passive 无 Cost/Cooldown）
  │     ├── 效果链上限检查
  │     ├── 冷却 ID 冲突检查
  │     └── 依赖图循环检查（Effect → Ability → Effect）
  │
  ├── Register (注入 DefRegistry<AbilityDef>)
  │
  └── Freeze (管线完成后不可变)
```

### 插件加载顺序

AbilityDef 引用所有其他 L1 Def，因此 `AbilityDefPlugin` 必须**最后加载**——在所有其他 L1 DefPlugin 完成注册之后：

```rust
// app.rs 中的加载顺序（严格的注册顺序）
fn build_content_pipeline(app: &mut App) {
    // L0 Vocabulary（基础词汇）
    app.add_plugins(TagDefPlugin);
    app.add_plugins(AttributeDefPlugin);

    // L1 Capability（按依赖顺序，被引用的先注册）
    app.add_plugins(StackingDefPlugin);     // 无 L1 同层依赖
    app.add_plugins(ConditionDefPlugin);    // 依赖 L0 Tag/Attribute
    app.add_plugins(ExecutionDefPlugin);    // 依赖 L0 Tag/Attribute
    app.add_plugins(CueDefPlugin);          // 依赖 ConditionDef
    app.add_plugins(ModifierDefPlugin);     // 依赖 L0 Tag/Attribute
    app.add_plugins(EffectDefPlugin);       // 依赖上述所有
    app.add_plugins(TriggerDefPlugin);      // 依赖 ConditionDef + AbilityDef
    app.add_plugins(TargetingDefPlugin);    // 依赖 ConditionDef

    // AbilityDef 最后加载——组合终端
    app.add_plugins(AbilityDefPlugin);      // 依赖所有 L1 Def
}
```

> **注意**: TriggerDef 也需要 AbilityDef 引用，导致双向引用依赖（TriggerDef → AbilityDef → TriggerDef）。这不是循环依赖——TriggerDef 和 AbilityDef 在同层注册后可交叉引用，注册管线中的依赖检查只校验存在性而非注册顺序。但 `AbilityDefPlugin` 的校验时机必须在 `TriggerDefPlugin` 注册完成后。

---

## 4. 校验规则

AbilityDef 是所有 Def 中引用最广、校验规则最多的类型。

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | AbilityId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1，不兼容版本拒绝加载 |
| V3 | `max_level >= 1` | 最大等级至少为 1（默认为 1） |
| V4 | `max_level <= 100` | 最大等级不超过 100 |
| V5 | `effect_chain` 不能为空 | 能力必须至少包含一个效果节点 |
| V6 | `effect_chain` 长度不超过 10 | 效果链长度上限禁止运行时修改 |
| V7 | `EffectNodeDef.delay_frames` >= 0 (如果设置) | 执行延迟不能为负 |
| V8 | `EffectNodeDef.chance` 范围 (0.0, 1.0] (如果设置) | 触发概率必须在有效范围内 |
| V9 | `EffectOverride` 参数合法 | 所有 override 覆盖值必须为正数 |
| V10 | `targeting.max_targets >= 1` | 目标数至少为 1 |
| V11 | `targeting.range` (如果设置) >= 0 | 射程不能为负 |
| V12 | `targeting.min_range` (如果设置) >= 0 | 最小射程不能为负 |
| V13 | `targeting.min_range <= range` (两者都设置时) | 射程下界不能超过上界 |
| V14 | `targeting.require_los` 与 `ignore_obstacles` 互斥 | 需要视野时不应忽略障碍物 |
| V15 | `CooldownDef.turns > 0` | 冷却回合数必须为正 |
| V16 | `LevelScaling` 中 `breakpoints[].level` 不重复 | 等级突破点等级不能重复 |
| V17 | `LevelScaling` 中 `breakpoints[].level` 在 [1, max_level] 范围内 | 突破点等级必须在有效范围内 |

### 4.2 AbilityType 约束校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | AbilityType::Passive 时 `costs` 必须为空 | 被动技能不能有消耗 |
| V19 | AbilityType::Passive 时 `cooldown` 必须为 None | 被动技能不能有冷却 |
| V20 | AbilityType::Passive 时 `triggers` 必须非空或有其他激活路径 | 被动技能需要触发条件 |
| V21 | AbilityType::Active 时必须有至少一个 `activation_conditions` 或 `activation_conditions` 可为空（无条件技能） | 主动技能条件可选 |
| V22 | AbilityType::Reaction 时 `triggers` 必须非空 | 反应技能必须关联 Trigger |
| V23 | AbilityType::Innate 时 `restrictions` 必须为 None | 内在能力不可有限制条件 |
| V24 | ActivationType::Instant 时无法 `interruptible` | 瞬发技能不可被打断 |
| V25 | ActivationType::Concentration 时 `interruptible` 必须为 true | 专注技能必须可打断 |

### 4.3 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V26 | `activation_conditions` 中的每个 ConditionId 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V27 | `triggers` 中的每个 TriggerId 已注册 | 在 DefRegistry<TriggerDef> 中存在 |
| V28 | `effect_chain[].effect_def_id` 中的每个 EffectId 已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V29 | `effect_chain[].condition` (如果设置) 中的 ConditionId 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V30 | `effect_chain[].cues` (如果设置) 中的每个 CueId 已注册 | 在 DefRegistry<CueDef> 中存在 |
| V31 | `stacking` (如果设置) 中的 StackingId 已注册 | 在 DefRegistry<StackingDef> 中存在 |
| V32 | `targeting.exclude_condition` (如果设置) 中的 ConditionId 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V33 | `targeting.filter_condition` (如果设置) 中的 ConditionId 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V34 | `costs[].resource_attribute` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V35 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V36 | `LevelScaling.cost_per_level` 中的 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V37 | `LevelScaling.breakpoints[].additional_effects` 中的每个 EffectId 已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V38 | `restrictions.required_class` (如果设置) 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V39 | `restrictions.required_race` (如果设置) 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V40 | `restrictions.prerequisite_abilities` (如果设置) 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V41 | `CooldownDef.cooldown_tag` (如果设置) 中的 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V42 | AbilityDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

### 4.4 引用递归校验

通过 Effect → Execution → Modifier → Attribute 的递归引用链，所有间接引用的 Def 也必须存在：

| # | 规则 | 说明 |
|---|------|------|
| V43 | `effect_chain` 中每个 EffectDef 的 `execution_def` (如果设置) 对应的 ExecutionDef 已注册 | 递归校验 2 层 |
| V44 | `effect_chain` 中每个 EffectDef 的 `modifier_defs` (如果设置) 中的 ModifierDef 已注册 | 递归校验 2 层 |
| V45 | `effect_chain` 中每个 EffectDef 的 `stacking_def` (如果设置) 对应的 StackingDef 已注册 | 递归校验 2 层 |
| V46 | `effect_chain` 中每个 EffectDef 的 `application_condition` (如果设置) 对应的 ConditionDef 已注册 | 递归校验 2 层 |
| V47 | `effect_chain` 中每个 EffectDef 的 `cues[].cue_def_id` 对应的 CueDef 已注册 | 递归校验 2 层 |
| V48 | `effect_chain` 中每个 EffectDef 的 `modifiers[].target_attribute` 对应的 AttributeDef 已注册 | 递归校验 3 层 |

### 4.5 依赖图校验

| # | 规则 | 说明 |
|---|------|------|
| V49 | EffectDef 不得直接或间接引用本 AbilityDef | 效果不可引用其所属技能（Effect → Ability → Effect 循环） |
| V50 | TriggerDef 的目标 AbilityDef 不得形成循环激活 | A 技能的 Trigger 激活 B 技能，B 技能的 Trigger 不能激活 A 技能 |
| V51 | `restrictions.prerequisite_abilities` 不得形成前置循环 | A 需要 B 前置，B 需要 A 前置，不允许 |
| V52 | `triggers` 引用的 TriggerDef 的目标不得为自身 | 技能不能通过 Trigger 自触发（无限循环） |

### 4.6 冷却与冲突检测

| # | 规则 | 说明 |
|---|------|------|
| V53 | `shared_cooldown_group` 同组内所有技能的 `cooldown_tag` 一致 | 共享冷却组的冷却 Tag 必须统一 |
| V54 | 具有相同 `shared_cooldown_group` 的多个 AbilityDef 不能存在冷却冲突 | 同组共享冷却的技能冷却时间差异不超过 50% |
| V55 | 同一 AbilityDef 不能在 `tags` 和 `cooldown_tag` 中使用相同 Tag 表示不同语义 | Tag 分类清晰 |

### 4.7 内联结构校验

| # | 规则 | 说明 |
|---|------|------|
| V56 | `LevelScaling.breakpoints` 至少需要 1 个 breakpoint (如果设置) | 等级突破数组非空 |
| V57 | `TargetingConfig` 中 Single 形状时 `max_targets` 应为 1 | 单体目标最多选 1 个 |
| V58 | `TargetingConfig` 中 `shape` 为 Area/Cone 时 `range` 必须设置 | 范围技能必须有射程 |
| V59 | `CostDef.amount` 不能为 0 (除非 optional = true) | 非可选消耗必须有正数消耗量 |
| V60 | `AbilityMetadata.interruptible` 仅对 Active 类型的 Concentration/Charge 生效 | 只有专注/蓄力技能可被打断 |

---

## 5. RON 示例

### 示例：火球术（Active / 瞬发 / AOE 伤害）

```ron
// AbilityDef 示例：火球术
//
// 一个标准 AOE 伤害技能，发射火球在目标区域爆炸造成火焰伤害。
// 引用预先注册的 EffectDef/Targeting/Condition/Stacking/Cue。
//
// 依赖的 Def:
//   - eff:fireball_explosion   → EffectDef（火焰爆发伤害 + 灼烧 DOT）
//   - eff:burning              → EffectDef（持续火焰伤害，3 回合）
//   - eff:small_explosion      → EffectDef（等级突破新增效果）
//   - tgt:aoe_radius_2         → TargetingDef（2 格半径 AOE）
//   - cond:has_mana            → ConditionDef（检查法力值是否足够）
//   - stk:unstackable          → StackingDef（不可堆叠）
//   - tag:damage_type_fire     → TagDef（火焰伤害类型）
//   - attr:mana                → AttributeDef（法力属性）

(
    id: "abl:fireball",
    name_key: "ability.abl_fireball.name",
    description_key: "ability.abl_fireball.desc",
    schema_version: 1,

    // 主动技能，瞬发
    ability_type: Active(
        activation: Instant,
    ),

    max_level: 5,

    // 消耗 30 点法力
    costs: [
        (
            resource_attribute: "attr:mana",
            amount: Fixed(30.0),
            optional: false,
            cost_type: Fixed,
        ),
    ],

    // 冷却 2 回合，执行完毕后开始计时
    cooldown: Some((
        turns: 2,
        starts_on_activate: false,
        shared_cooldown_group: None,
        cooldown_tag: None,
    )),

    // 必须有足够法力值才能施放
    activation_conditions: [
        "cond:has_mana",
    ],

    // 主动技能无 Trigger
    triggers: [],

    // 目标选择：敌人、2 格半径 AOE、最大射程 8
    targeting: (
        target_type: Enemy,
        shape: Area(
            radius: 2.0,
        ),
        range: Some(8.0),
        max_targets: 6,
        include_self: false,
        exclude_condition: Some("cond:has_fire_immunity"),
        require_los: true,
        ignore_obstacles: false,
        allow_dead_targets: false,
    ),

    // 效果链：
    //   1. 火焰爆发伤害（主要命中伤害）
    //   2. 灼烧 DOT（3 回合火焰持续伤害，30% 概率触发）
    effect_chain: [
        // 节点 1：火焰爆发伤害
        (
            effect_def_id: "eff:fireball_explosion",
            override_params: Some((
                duration_override: None,
                magnitude_override: Some(PerLevel(
                    base: 8.0,
                    per_level: 2.0,
                )),
                period_override: None,
            )),
            delay_frames: None,
            chance: None,
        ),
        // 节点 2：灼烧 DOT（30% 概率附加）
        (
            effect_def_id: "eff:burning",
            override_params: Some((
                duration_override: Some(Fixed(3.0)),
                magnitude_override: Some(Fixed(2.0)),
                period_override: Some(1),
            )),
            delay_frames: Some(5),
            chance: Some(0.3),
            condition: Some("cond:has_no_fire_immunity"),
        ),
    ],

    // 不可堆叠（同时只能激活一个火球术）
    stacking: Some("stk:unstackable"),

    // 技能等级缩放：每级 +2 伤害，3 级时获得爆炸溅射效果
    level_scaling: Some((
        damage_per_level: Some(Fixed(2.0)),
        range_per_level: Some(0.5),
        cost_per_level: Some([
            ("attr:mana", Fixed(5.0)),
        ]),
        cooldown_reduction_per_level: None,
        breakpoints: [
            (
                level: 3,
                description_key: "ability.abl_fireball.breakpoint.3",
                additional_effects: ["eff:small_explosion"],
            ),
        ],
    )),

    // 学习限制：法师职业
    restrictions: Some((
        required_class: Some(["tag:class_mage"]),
        min_level: Some(1),
        required_attributes: Some([
            (
                attribute_id: "attr:intelligence",
                operator: GreaterOrEqual,
                threshold: 12.0,
            ),
        ]),
        prerequisite_abilities: None,
    )),

    // 标签
    tags: ["tag:damage_type_fire", "tag:class_mage", "tag:combat"],

    // 元数据
    metadata: (
        visible: true,
        interruptible: false,
        icon_key: Some("icon_skill_fireball"),
        cast_animation: Some("anim_cast_fireball"),
    ),
)
```

### 示例：元素护盾（Passive / 常驻 / 防御 Buff）

```ron
// AbilityDef 示例：元素护盾
//
// 被动技能——常驻火焰抗性 +10%。
// 无消耗、无冷却、常驻生效。
//
// 依赖的 Def:
//   - eff:fire_resistance      → EffectDef（火焰抗性 +10%）
//   - cond:is_combat           → ConditionDef（仅在战斗中生效）
//   - tag:damage_type_fire     → TagDef

(
    id: "abl:elemental_shield",
    name_key: "ability.abl_elemental_shield.name",
    description_key: "ability.abl_elemental_shield.desc",
    schema_version: 1,

    ability_type: Passive,
    max_level: 1,

    // 被动技能无消耗
    costs: [],

    // 被动技能无冷却
    cooldown: None,

    // 战斗中常驻生效
    activation_conditions: [
        "cond:is_combat",
    ],

    // 被动技能不依赖触发事件（通过 Effect 自身条件控制）
    triggers: [],

    targeting: (
        target_type: Self_,
        shape: Single,
        max_targets: 1,
        include_self: true,
        require_los: false,
        ignore_obstacles: true,
        allow_dead_targets: false,
    ),

    effect_chain: [
        (
            effect_def_id: "eff:fire_resistance",
            delay_frames: None,
            chance: None,
        ),
    ],

    stacking: Some("stk:unstackable"),

    tags: ["tag:class_mage", "tag:defensive", "tag:passive"],

    metadata: (
        visible: true,
        interruptible: false,
        icon_key: Some("icon_skill_elemental_shield"),
        cast_animation: None,
    ),
)
```

### 示例：反击（Reaction / 回合外响应）

```ron
// AbilityDef 示例：反击
//
// 反应技能——受到近战攻击时自动对攻击者造成 50% 伤害。
//
// 依赖的 Def:
//   - trg:on_melee_attacked    → TriggerDef（受到近战攻击时触发）
//   - eff:counter_attack       → EffectDef（反击伤害 = 50% 攻击伤害）
//   - cond:has_weapon          → ConditionDef（必须装备武器才能反击）
//   - stk:unstackable          → StackingDef

(
    id: "abl:counter_attack",
    name_key: "ability.abl_counter_attack.name",
    description_key: "ability.abl_counter_attack.desc",
    schema_version: 1,

    ability_type: Reaction(
        activation: Instant,
    ),
    max_level: 1,

    // 反应技能可能有消耗（如体力）
    costs: [],

    cooldown: Some((
        turns: 1,
        starts_on_activate: true,
        shared_cooldown_group: None,
        cooldown_tag: None,
    )),

    activation_conditions: [
        "cond:has_weapon",
    ],

    // 反应技能必须关联 Trigger
    triggers: [
        "trg:on_melee_attacked",
    ],

    targeting: (
        target_type: Enemy,
        shape: Single,
        range: Some(1.5),
        max_targets: 1,
        include_self: false,
        require_los: false,
        ignore_obstacles: false,
        allow_dead_targets: false,
    ),

    effect_chain: [
        (
            effect_def_id: "eff:counter_attack",
            delay_frames: None,
            chance: None,
        ),
    ],

    stacking: Some("stk:unstackable"),

    tags: ["tag:combat", "tag:reaction", "tag:warrior"],

    metadata: (
        visible: true,
        interruptible: false,
        icon_key: Some("icon_skill_counter_attack"),
        cast_animation: Some("anim_counter_attack"),
    ),
)
```

---

## 6. 与其他 Def 的关系

| 对比维度 | AbilityDef | EffectDef |
|----------|-----------|-----------|
| 本质 | 编排者——编排 Condition/Cost/Targeting/Effect | 执行者——定义效果做什么、持续多久 |
| 是否引用其他 Def | 是——引用所有 L1 Def | 是——引用 Modifier/Condition/Execution/Stacking/Cue |
| 是否可独立注册 | 是 | 是 |
| 是否组合终端 | 是（L1 层最终组合） | 否（可被 AbilityDef 和 BuffDef 引用） |
| 运行时实例 | AbilityInstance（ECS Component） | ActiveEffect（ECS Component） |
| 典型存储 | `abilities.ron` | `effects.ron` |

| 对比维度 | AbilityDef | BuffDef |
|----------|-----------|---------|
| 本质 | 一个完整的能力（主动/被动/反应） | 一个持久状态的 Effect 容器 |
| 生命周期 | Ready → Casting → Active → Cooldown → Ready | Applied → Tick → Expired → Removed |
| 激活方式 | 手动/触发/常驻 | 由 Effect 应用时自动创建 |
| 依赖关系 | 引用 EffectDef | 包装 EffectDef |

---

## 7. 设计决策记录

| # | 决策 | 理由 |
|---|------|------|
| D1 | **Targeting 内联而非引用 TargetingDef** | 目标选择紧密耦合于具体技能，跨技能复用价值低。内联减少一次性 Def 的膨胀 |
| D2 | **Cost 和 Cooldown 内联定义** | 消耗和冷却的数据结构简单，且与技能特有数值强关联，不适合独立注册 |
| D3 | **effect_chain 使用 Vec<EffectNodeDef> 而非单个 Effect** | 支持效果链（多个 Effect 按顺序执行），每个节点可独立控制条件/延迟/概率 |
| D4 | **AbilityType 合并 schema 中的 AbilityCategory + ActivationType** | 减少枚举层级，让 AbilityType 同时表达"大类"和"激活方式" |
| D5 | **activation_conditions 使用 Vec<ConditionId> 而非 Option<Condition>** | 支持多个条件的 AND 检查，且统一使用 ConditionDef 引用（避免内联/引用混用） |
| D6 | **triggers 仅 Passive/Reaction 类型有效** | Active 技能有 PlayerInput 系统处理激活，不需要 Trigger 监听 |
| D7 | **LevelScaling 以 Option 形式嵌入而非独立 Def** | 等级缩放数据与技能强关联，且结构简单不适合独立注册 |
| D8 | **AbilityId 类型前缀 "abl:"** | 统一 Def ID 命名规范，与其他 Def 类型（eff:/tgt:/cond:）区分 |
