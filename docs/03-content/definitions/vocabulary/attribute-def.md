---
id: 03-content.definitions.vocabulary.attribute-def
title: AttributeDef — Attribute Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# AttributeDef — Attribute Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/capabilities/attribute_domain.md` | **数据 Schema**: `docs/04-data/capabilities/attribute_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

AttributeDef 定义了一个**可量化的数值属性**的元数据——属性是什么、属于哪个类别、默认值是多少、取值范围是什么。

属性是游戏角色/实体的**核心数值维度**：
- HP（生命值）、MP（法力值）、ATK（攻击力）、DEF（防御力）
- STR（力量）、DEX（敏捷）、CON（体质）、INT（智力）
- Move Range（移动力）、Action Points（行动力）
- Speed（速度）、Accuracy（命中率）、Evasion（闪避率）

### AttributeDef 的定位

AttributeDef 不存储属性值——值存储在运行时 `AttributeContainer`（ECS Component）中。AttributeDef 只定义属性的**身份和元数据**：

> AttributeDef = "力量属性是什么"
> AttributeContainer = "角色 A 的力量值是 18"

这种 Definition/Instance 分离是 Content Platform 的核心原则。

### 与 data Schema 的关系

| 概念 | 归属层 | 说明 |
|------|--------|------|
| `AttributeId` | Content (03-content) | 属性唯一标识符（如 `attr:strength`） |
| `AttributeDef` | Content (03-content) | 属性元数据 Asset：ID + 类别 + 边界 |
| `AttributeDefinition` | Data Schema (04-data) | 属性的完整定义（含 DerivedFormula 引用等） |
| `AttributeValue` | Data Schema (04-data) | 运行时数值（base + current 分离） |
| `AttributeContainer` | Data Schema (04-data) | ECS 组件：实体上的属性集合 |
| `DerivedFormula` | Data Schema (04-data) | 派生属性计算公式 |

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `attribute_domain.md` | 属性分类体系（Primary/Secondary/Derived/Resource）、派生公式规则 |
| `attribute_schema.md` | AttributeValue、DerivedFormula、AttributeContainer 完整结构 |
| `effect-def.md` | 本 Def 被 EffectDef 的 modifier 引用 |
| `modifier-def.md` | 本 Def 被 ModifierDef.target 引用 |
| `condition-def.md` | 本 Def 被 ConditionDef 的 AttributeCheck 引用 |
| `execution-def.md` | 本 Def 被 ExecutionDef 的伤害/治疗公式引用 |
| `character-def.md` | 本 Def 被 CharacterDef.base_attributes 引用 |
| `monster-def.md` | 本 Def 被 MonsterDef.base_attributes 引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Attribute Def 定义——可量化数值属性的元数据。
///
/// AttributeDef 只定义属性的身份、类别、数值范围，不存储属性的运行时值。
/// 属性值由 Instance 层的 AttributeContainer 管理。
///
/// 属性无默认运算逻辑——具体公式（如"最大HP = 体质 × 10"）在
/// L3 ProgressionDef 或 Data Schema 的 DerivedFormula 中定义。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct AttributeDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: AttributeId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 属性分类 ──
    /// 属性分类：Primary / Secondary / Derived / Resource
    pub category: AttributeCategory,

    // ── 数值边界 ──
    /// 默认值——属性未被任何系统显式设置时的基础值
    ///
    /// 对于 Primary 属性，这是角色的起始值。
    /// 对于 Derived 属性，这个值可能被公式覆盖。
    pub default_value: f32,

    /// 最小值（可选）——None 表示无下限
    pub min_value: Option<f32>,

    /// 最大值（可选）——None 表示无上限
    pub max_value: Option<f32>,
}
```

### 内嵌枚举

```rust
/// 属性分类——标识属性的业务角色
///
/// 分类决定了属性的计算方式和使用场景，但不影响运行时存储结构。
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AttributeCategory {
    /// 主属性——决定角色基本能力的核心属性
    ///
    /// 示例: 力量、敏捷、体质、智力、感知、魅力
    /// 通常由创建角色时分配，通过升级获得少量增长。
    Primary,
    /// 副属性——由主属性推算的衍生属性
    ///
    /// 示例: 熟练加值（Proficiency Bonus）、先攻调整值
    /// 通常不直接分配，由公式计算得到。
    Secondary,
    /// 派生属性——由多属性综合计算得出
    ///
    /// 示例: 最大生命值（MaxHP = 体质×10 + 等级修正）、防御等级（AC）
    /// 计算规则在 L3 ProgressionDef 或 Schema DerivedFormula 中。
    Derived,
    /// 资源属性——可消耗的资源量
    ///
    /// 示例: 当前生命值（HP）、法力值（MP）、行动力（AP）
    /// 经常变化，需要 base/current 分离存储。
    Resource,
}
```

### 字段说明

- **`category`**: 分类决定属性在计算体系中的角色。Primary 是输入的起点，Secondary 是中间推导，Derived 是综合结果，Resource 是运行时状态
- **`default_value`**: 初始默认值。对于 Primary 属性，这是角色在无任何调整时的基础值。对于 Resource 属性，default_value 通常等于最大值
- **`min_value`/`max_value`**: 数值边界。`None` 表示无边界（如 Primary 属性通常无上限，Resource 属性通常有最小值 0 但无系统上限）。边界校验在属性修改时由 AttributeContainer 执行，不在 Def 层校验

### 无 tags 字段（L0 约束）

AttributeDef 不包含 `tags: Vec<TagId>` 字段，因为 L0 Def 禁止同层引用（TagDef 是 L0 Def）。属性分类通过 `category` 枚举代替标签系统。

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// AttributeDef 通过 L0 VocabularyPlugin 批量注册
pub fn get_attribute_def(
    id: &AttributeId,
    registry: &DefRegistry<AttributeDef>,
) -> Option<&AttributeDef> {
    registry.get(id)
}

/// 按分类过滤 AttributeDef
pub fn get_attribute_defs_by_category(
    category: AttributeCategory,
    registry: &DefRegistry<AttributeDef>,
) -> Vec<&AttributeDef> {
    registry.iter()
        .filter(|def| def.category == category)
        .collect()
}
```

### 注册生命周期

```
Load (attributes.ron) → Deserialize → Validate → Register (DefRegistry<AttributeDef>) → Freeze
```

### 不存储集合关系

AttributeDef 的 Registry 仅提供基础查询（按 ID、按类别）。属性集合的运行时关系（"哪些属性构成了这个角色的攻击力"）由 L3 ProgressionDef 或 Instance 层的 AttributeContainer 管理，不在 Def 层表达。

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | AttributeId 不能为空字符串 |
| V2 | `id` 格式合法 | 必须匹配 `^attr:[a-z][a-z0-9_]+$`（如 `attr:strength`、`attr:max_hp`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 属性必须有显示名称 |
| V5 | `description_key` 非空 | 属性必须有描述文本 |
| V6 | `category` 为有效枚举值 | 必须是 AttributeCategory 的四个变体之一 |
| V7 | `default_value` 合法 | 非 NaN、非 Infinite |
| V8 | `min_value` 合法（若设置） | 非 NaN、非 Infinite |
| V9 | `max_value` 合法（若设置） | 非 NaN、非 Infinite |
| V10 | `min_value` <= `max_value`（若两者均设置） | 数值范围不能颠倒 |
| V11 | `default_value` 在 `[min, max]` 范围内（若边界设置） | 默认值不能超出边界 |

### 4.2 分类级校验

| # | 规则 | 说明 |
|---|------|------|
| V12 | Resource 类属性应设置 `min_value` | 资源属性通常有最小值（通常是 0） |
| V13 | Primary 类属性通常无 `max_value` | 主属性通常无硬上限，但此规则为建议（warning 级别） |
| V14 | Derived 类属性的 `default_value` 可能被覆盖 | 校验器应检查是否有对应的 DerivedFormula 定义（依赖性检查在 L3 加载时进行） |

### 4.3 无跨 Def 引用校验（L0 约束）

AttributeDef 是 L0 Def，禁止引用任何其他 Def。因此没有引用存在性检查、没有跨层引用检查。

---

## 5. RON 示例

```ron
// AttributeDef 示例 — Primary 属性
(
    id: "attr:strength",
    name_key: "attr.strength.name",
    description_key: "attr.strength.desc",
    schema_version: 1,

    category: Primary,

    default_value: 10.0,
    min_value: Some(1.0),
    max_value: None,
)
```

```ron
// AttributeDef 示例 — Resource 属性
//
// Resource 属性通常有 min=0 和明确的 max_value。
(
    id: "attr:max_hp",
    name_key: "attr.max_hp.name",
    description_key: "attr.max_hp.desc",
    schema_version: 1,

    category: Derived,

    default_value: 100.0,
    min_value: Some(1.0),
    max_value: None,
)
```

```ron
// AttributeDef 示例 — Resource 属性
//
// Resource 总是有最小值和最大值，default 通常等于最大值。
(
    id: "attr:current_hp",
    name_key: "attr.current_hp.name",
    description_key: "attr.current_hp.desc",
    schema_version: 1,

    category: Resource,

    default_value: 100.0,
    min_value: Some(0.0),
    max_value: Some(100.0),
)
```

---

## 6. 设计说明

### 边界校验的位置

`min_value` 和 `max_value` 在 Def 层定义边界，但边界的**运行时强制**属于 AttributeContainer 的职责。Def 层的边界是一种"文档化约束"，Instance 层的边界才是"可执行约束"。两者可能不同——Instance 层可能有更严格的临时边界（如"在某个区域内最大 HP 减半"）。

### 为什么没有`unit`字段（如 "HP"、"MP"）？

属性单位（"点"、"%"）是 Localization 层的概念，不应在 Def 层硬编码。UI 显示时通过 LocalizationKey 上下文选择正确的单位格式。

### 属性基线 vs 属性公式

AttributeDef 定义了属性的**基线信息**（默认值、范围），但不定义属性的**计算方式**（派生公式、升级增长）。后者属于：
- `DerivedFormula`（Data Schema 层）——单个属性的计算规则
- `ProgressionDef`（L3 Gameplay）——角色成长的全局规则

这种分离允许在不变更 Def 的情况下修改属性计算逻辑。

---

*本文档由 @content-architect 维护。*
