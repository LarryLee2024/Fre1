---
id: 03-content.definitions.condition-def
title: ConditionDef — Condition Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ConditionDef — Condition Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/condition_domain.md` | **数据 Schema**: `docs/04-data/capabilities/condition_schema.md` | **插件代码**: `src/content/plugins/condition_plugin.rs`

---

## 1. Overview

ConditionDef 是**可命名的条件检查表达式**——定义一个"判断是否允许"的规则，可在多个场景中按 ID 引用。

Condition 是贯穿全系统的**统一条件检查语言**：
- 技能激活前检查：条件不满足则技能不可用
- 效果应用前检查：条件不满足则效果不应用
- 装备穿戴检查：条件不满足则装备不可穿戴
- Buff 持续检查：条件不满足则 Buff 暂停
- 对话分支过滤：条件不满足则对话选项不显示
- 任务完成检查：条件不满足则任务不可完成

### 设计原则

- **ConditionDef 是可命名的 Condition**：数据 Schema 中的 Condition 是内联表达式，ConditionDef 将其包装为可注册、可引用的 Asset
- **ConditionDef 是纯函数**：评估过程无副作，输入实体状态 -> 输出 Pass/Fail
- **组合优于继承**：ConditionGroup 支持 AND/OR/NOT 任意嵌套，无需定义复杂的条件继承体系

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `condition_domain.md` | 条件组合规则、免疫最高优先级、自定义条件注册 |
| `condition_schema.md` | Condition 枚举、TagRequirement、AttributeCheck、ConditionGroup 的完整类型定义 |
| `effect-def.md` | 本 Def 被 EffectDef.application_condition 引用 |
| `trigger-def.md` | 本 Def 被 TriggerDef.condition 引用 |
| `targeting-def.md` | 本 Def 被 TargetingDef.filter_condition / exclude_condition 引用 |
| `buff-def.md` | 本 Def 被 BuffDef.condition 引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Condition Def 定义——可命名的条件检查表达式。
///
/// ConditionDef 包装一个 Condition 表达式树，使其可被注册、按 ID 引用、
/// 在多个场景中共享。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ConditionDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: ConditionId,
    /// 显示名称（本地化 Key，用于调试显示）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key，说明此条件检查什么）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 条件核心 ──
    /// 条件表达式（不可变，加载时校验后冻结）
    pub condition: Condition,

    // ── 元数据 ──
    /// 分类标签（用于过滤和组织）
    pub tags: Vec<TagId>,

    /// 所属领域（用于自定义条件的归属追踪）
    pub domain: Option<String>,
}
```

### 用于条件表达式的数据结构

ConditionDef 的 `condition` 字段使用 `Condition` 枚举，其定义见 `condition_schema.md`。核心结构概览：

```rust
/// 统一条件表达式
#[derive(Deserialize, Clone, Debug)]
pub enum Condition {
    /// 标签需求检查（单标签 Has/Not）
    TagRequirement(TagRequirement),
    /// 多标签匹配（TagQuery Any/All/None + 层级继承）
    TagMatch(TagQuery),
    /// 属性阈值检查
    AttributeCheck(AttributeCheck),
    /// 资源充足检查
    ResourceCheck(ResourceCheck),
    /// 条件组合（AND/OR/NOT）
    Group(ConditionGroup),
    /// 自定义条件（由 Domain 注册扩展）
    Custom(CustomConditionDef),
}
```

### 字段说明

- **`condition`**: 这是 ConditionDef 的核心负载——一个完整的 Condition 表达式树。可以是一个简单的 TagRequirement，也可以是一个多层嵌套的 AND/OR/NOT 组合
- **`tags`**: 用于分类和过滤的条件标签，如 `tag:combat`、`tag:inventory`、`tag:dialogue`
- **`domain`**: 记录此条件属于哪个领域（如 "combat"、"spell"、"quest"）便于管理和冲突排查

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ConditionDef 注册插件
pub struct ConditionDefPlugin;

impl Plugin for ConditionDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ConditionDef>();
        app.init_asset_loader::<RonAssetLoader<ConditionDef>>();
        app.insert_resource(DefRegistry::<ConditionDef>::new());

        app.add_systems(
            PreUpdate,
            load_condition_defs
                .run_if(resource_changed::<Assets<ConditionDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 ConditionDef
pub fn get_condition_def(id: &ConditionId, registry: &DefRegistry<ConditionDef>) -> Option<&ConditionDef> {
    registry.get(id)
}
```

### 注册生命周期

```
Load (conditions.ron) → Deserialize → Validate → Register (DefRegistry<ConditionDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ConditionId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | 条件树深度不超过上限 | 默认最大嵌套深度 10 层（防止递归爆炸） |

### 4.2 条件表达式校验

| # | 规则 | 说明 |
|---|------|------|
| V4 | Condition::Group(Not) 必须且只能包含一个子条件 | NOT 逻辑的 conditions.len() == 1 |
| V5 | Condition::Group(All/Any) 必须包含至少一个子条件 | 空组合无意义 |
| V6 | AttributeCheck.threshold 合法 | 非 NaN，非 Infinite |
| V7 | ResourceCheck.required_amount >= 0 | 资源检查不能为负数 |
| V8 | ResourceCheck.resource_attribute 应与目标资源属性匹配 | 语义检查 |

### 4.3 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | TagRequirement.target_tags 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V10 | AttributeCheck.attribute_id 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V11 | AttributeCheck.source_attribute (如果设置) 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V12 | ResourceCheck.resource_attribute 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V13 | ConditionDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

---

## 5. RON 示例

```ron
// ConditionDef 示例：目标不是火焰免疫
//
// 简单的 TagRequirement 条件，检查目标是否拥有火焰免疫标签。
// 被 eff:burn 和 buff:burning 引用。
(
    id: "cond:has_no_fire_immunity",
    name_key: "condition.has_no_fire_immunity.name",
    description_key: "condition.has_no_fire_immunity.desc",
    schema_version: 1,

    condition: TagRequirement((
        mode: HasNone,
        target_tags: ["tag:immunity_fire"],
        respect_hierarchy: true,
    )),

    tags: ["tag:combat", "tag:spell"],
    domain: Some("combat"),
)
```

```ron
// ConditionDef 示例：战斗中的敌人且生命低于 30%
//
// 组合条件，用于触发斩杀类技能。
(
    id: "cond:enemy_low_hp_combat",
    name_key: "condition.enemy_low_hp_combat.name",
    description_key: "condition.enemy_low_hp_combat.desc",
    schema_version: 1,

    condition: Group((
        logic: All,
        conditions: [
            TagRequirement((
                mode: HasAll,
                target_tags: ["tag:faction_enemy"],
                respect_hierarchy: false,
            )),
            AttributeCheck((
                attribute_id: "attr:current_hp",
                operator: LessOrEqual,
                threshold: 0.0,
                source_attribute: Some("attr:max_hp"),
            )),
        ],
    )),

    tags: ["tag:combat", "tag:damage"],
    domain: Some("combat"),
)
```

---

## 6. 设计说明

### 内联 vs 引用

Condition 表达式在内容配置中有两种使用方式：

```ron
// 方式 A：内联定义（不经过 ConditionDef）
application_condition: Some(
    TagRequirement(
        mode: HasNone,
        target_tags: ["tag:immunity_fire"],
        respect_hierarchy: true,
    ),
),

// 方式 B：引用 ConditionDef（推荐用于跨 Def 共享的条件）
application_condition: Some("cond:has_no_fire_immunity"),
```

建议在以下场景使用 ConditionDef 引用：
- 同一条件被多个 Def 引用（如"目标是否存活"被所有技能引用）
- 复杂组合条件（避免在每个 Def 中重复编写相同的嵌套组合）
- 需要全局调整的条件规则（改 ConditionDef 一处生效）
