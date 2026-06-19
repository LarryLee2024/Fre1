---
id: 03-content.definitions.targeting-def
title: TargetingDef — Targeting Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# TargetingDef — Targeting Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/targeting_domain.md` | **数据 Schema**: `docs/04-data/capabilities/targeting_schema.md` | **插件代码**: `src/content/plugins/targeting_plugin.rs`

---

## 1. Overview

TargetingDef 定义**目标选择规则**——一个技能/效果作用于哪些实体、以什么形状选择目标、受哪些限制和过滤条件。Targeting 是能力系统与战场之间的桥梁：

- 单体近战攻击：Enemy + Single + Range(1.5)
- 火球术范围伤害：Enemy + Area(radius=2) + Range(10)
- 友方治疗：Ally + Single + Range(5) + Priority(LowestHealth)
- 连锁闪电：Enemy + Chain(bounces=3) + Range(8)
- 直线冰枪：Enemy + Line(length=6, width=1) + Range(8)

### 设计原则

- **TargetingDef 只做选择，不做表现**：它不关心目标被选中后的视觉效果或反馈
- **Locational + Conditional 组合**：TargetShape 决定"范围"，FilterCondition 决定"在范围内选谁"
- **可被 Ability 和 Effect 引用**：既是技能的目标选择方案，也可是效果链中某些效果的独立目标选择

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `targeting_domain.md` | TargetType 分类规则、网格距离计算、视野和掩体规则 |
| `targeting_schema.md` | TargetingDef 完整字段、TargetType、TargetShape、PriorityRule 枚举定义 |
| `condition-def.md` | 本 Def 的 `filter_condition` / `exclude_condition` 字段 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Targeting Def 定义——目标选择规则。
///
/// 定义技能/效果作用于哪些实体、以什么形状选择目标、受哪些限制。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct TargetingDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: TargetingId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 目标类别与形状 ──
    /// 目标类别（自身/友方/敌方/所有等）
    pub target_type: TargetType,

    /// 范围形状（单体/圆形/直线/锥形/链式/爆炸/墙体）
    pub shape: TargetShape,

    // ── 范围限制 ──
    /// 最大射程（网格单位，None = 无限制）
    pub range: Option<f32>,

    /// 最小射程（None = 无限制）
    pub min_range: Option<f32>,

    /// 最大目标数
    pub max_targets: u32,

    // ── 自引用与排除 ──
    /// 是否允许选择施法者自身
    pub include_self: bool,

    /// 排除条件（满足此条件的目标不被选中）
    pub exclude_condition: Option<ConditionDefId>,

    /// 附加过滤条件（只选中满足此条件的目标）
    pub filter_condition: Option<ConditionDefId>,

    // ── 视野与环境 ──
    /// 是否需要视野
    pub require_los: bool,

    /// 是否忽略障碍物
    pub ignore_obstacles: bool,

    /// 能否选择已死亡实体
    pub allow_dead_targets: bool,

    // ── 自动选择 ──
    /// 优先级排序规则（多个可选目标时的自动选择）
    pub priority_rule: Option<PriorityRule>,

    // ── 元数据 ──
    /// 分类标签
    pub tags: Vec<TagId>,
}

/// 目标类别
#[derive(Deserialize, Clone, Debug)]
pub enum TargetType {
    Self_,
    Ally,
    Enemy,
    Dead,
    Neutral,
    Any,
    Summon,
    Party,
    Custom(String),
}

/// 范围形状
#[derive(Deserialize, Clone, Debug)]
pub enum TargetShape {
    Single,
    Area { radius: f32 },
    Line { length: f32, width: f32 },
    Cone { length: f32, angle: f32 },
    Chain { bounces: u32, bounce_range: f32, allow_retarget: bool },
    Burst { center_radius: f32, burst_radius: f32 },
    Wall { length: f32, width: f32 },
}

/// 自动选择优先级规则
#[derive(Deserialize, Clone, Debug)]
pub enum PriorityRule {
    Nearest,
    Farthest,
    LowestHealth,
    HighestHealth,
    HighestAttribute(AttributeId),
    LowestAttribute(AttributeId),
    Random,
    Custom(String),
}
```

### 字段说明

- **`target_type` + `shape`**: Targeting 的核心组合。TargetType 定义"选谁"，TargetShape 定义"什么范围"。如单目标治疗 = Ally + Single，AOE 攻击 = Enemy + Area
- **`range` / `min_range`**: 以网格单位计量的射程。range = 近战范围(1.5)，range = 远程范围(10)。min_range 用于"无法攻击最近的目标"场景
- **`include_self`**: 当 target_type = Ally 或 Any 时，是否包括施法者自己。通常治疗要 include_self，AOE 要 exclude_self
- **`exclude_condition` / `filter_condition`**: 目标的二次筛选。exclude: "排除火焰免疫目标"。filter: "只选择持有燃烧标签的目标"
- **`require_los`**: 是否需要直线视野（用于掩体系统）。近战通常不需要，远程需要
- **`priority_rule`**: 自动选择时的排序规则（如"优先选择血量最低的友方目标"用于自动治疗）

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// TargetingDef 注册插件
pub struct TargetingDefPlugin;

impl Plugin for TargetingDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<TargetingDef>();
        app.init_asset_loader::<RonAssetLoader<TargetingDef>>();
        app.insert_resource(DefRegistry::<TargetingDef>::new());

        app.add_systems(
            PreUpdate,
            load_targeting_defs
                .run_if(resource_changed::<Assets<TargetingDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 TargetingDef
pub fn get_targeting_def(id: &TargetingId, registry: &DefRegistry<TargetingDef>) -> Option<&TargetingDef> {
    registry.get(id)
}

/// 按 TargetType 过滤
pub fn get_targeting_by_type(target_type: TargetType, registry: &DefRegistry<TargetingDef>) -> Vec<&TargetingDef> {
    registry.iter()
        .filter(|def| matches_target_type(&def.target_type, &target_type))
        .collect()
}
```

### 注册生命周期

```
Load (targeting.ron) → Deserialize → Validate → Register (DefRegistry<TargetingDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | TargetingId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `max_targets >= 1` | 目标数至少为 1 |
| V4 | 形状参数合法 | Area.radius > 0, Line.length > 0, Cone.length > 0, Cone.angle 在 (0, 360] |
| V5 | `range` (如果设置) >= 0 | 射程不能为负 |
| V6 | `min_range` (如果设置) >= 0 | 最小射程不能为负 |
| V7 | `min_range <= range` (两者都设置时) | 射程下界不能超过上界 |
| V8 | Single 形状时 `max_targets` 应为 1 | 单体目标最多选 1 个 |
| V9 | Chain.bounces >= 1 | 链式弹射至少弹射 1 次 |
| V10 | `require_los` 与 `ignore_obstacles` 互斥 | 需要视野时不应忽略障碍物 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V11 | `exclude_condition` (如果设置) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V12 | `filter_condition` (如果设置) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V13 | `priority_rule.HighestAttribute/LowestAttribute` 中的 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V14 | TargetingDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

---

## 5. RON 示例

```ron
// TargetingDef 示例：圆锥形 AOE
//
// 用于火焰吐息、锥形冰冻等技能。
(
    id: "tgt:cone_3",
    name_key: "targeting.cone_3.name",
    description_key: "targeting.cone_3.desc",
    schema_version: 1,

    target_type: Enemy,
    shape: Cone(
        length: 3.0,
        angle: 90.0,
    ),

    range: Some(3.0),
    max_targets: 6,

    include_self: false,
    require_los: true,
    ignore_obstacles: false,
    allow_dead_targets: false,

    tags: ["tag:combat", "tag:aoe"],
)
```

```ron
// TargetingDef 示例：单体友方治疗
//
// 用于治疗术、恢复等技能。
(
    id: "tgt:single_ally_heal",
    name_key: "targeting.single_ally_heal.name",
    description_key: "targeting.single_ally_heal.desc",
    schema_version: 1,

    target_type: Ally,
    shape: Single,

    range: Some(5.0),
    max_targets: 1,

    include_self: true,
    exclude_condition: Some("cond:target_at_full_hp"),
    require_los: true,
    allow_dead_targets: false,

    priority_rule: Some(LowestHealth),

    tags: ["tag:combat", "tag:heal"],
)
```
