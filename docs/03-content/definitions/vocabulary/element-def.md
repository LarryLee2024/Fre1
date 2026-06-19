---
id: 03-content.definitions.vocabulary.element-def
title: ElementDef — Element Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ElementDef — Element Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/domains/combat_domain.md`, `docs/02-domain/capabilities/attribute_domain.md` | **数据 Schema**: `docs/04-data/domains/element_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

ElementDef 定义了游戏中的**元素属性**——一个与伤害类型（DamageType）正交的分类维度，用于表达元素之间的克制关系、亲和度加成、以及技能的元素归属。

元素系统是 SRPG 中常见的增伤/减伤机制层：
- Fire（火焰）元素克制 Ice（冰冻）— 火系技能对冰系目标造成额外伤害
- Lightning（闪电）克制 Water（水）— 电系技能对水浸目标造成额外伤害
- Holy（神圣）对 Undead（亡灵）有克制效果

### ElementDef 与 TagDef 的关系

ElementDef 存在于 L0 的原因正是 TagDef **无法表达元素的层次关系和克制矩阵**：

| 能力 | TagDef | ElementDef |
|------|--------|------------|
| 类型安全 ID | `TagId` | `ElementId` |
| 克制关系 | 无（TagDef 扁平） | L3 ElementInteractionMatrix |
| 抗性系统 | 通用标签过滤 | 专门的元素抗性计算 |
| 亲和度加成 | 无 | L3 元素亲和度公式 |

如果 TagDef 支持了层级关系和交互矩阵，ElementDef 可以被合并到 TagDef。但在当前设计中，TagDef 保持扁平，ElementDef 提供类型安全的元素标识。

### ElementDef 与 DamageTypeDef 的关系

两者是正交关系：

```
伤害实例 = DamageType + Element + Value

      DamageType (物理/魔法分类)     Element (元素属性)
         dmg:fire                     elem:fire
         dmg:slashing                 elem:none (物理技能通常无元素)
         dmg:ice                      elem:ice
```

一个火焰伤害（`dmg:fire`）通常具有火焰元素（`elem:fire`），但也可以被 L3 系统映射为具有其他元素属性（如"火法师使用冰霜法杖时，火焰伤害获得冰元素"）。

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `combat_domain.md` | 元素克制规则、元素亲和度计算 |
| `element_schema.md` | ElementInteractionMatrix、ElementResistance 的数据结构 |
| `effect-def.md` | 本 Def 被 EffectDef.element 引用（技能的元素归属） |
| `execution-def.md` | 本 Def 被 ExecutionDef 的元素缩放公式引用 |
| `buff-def.md` | 本 Def 被 BuffDef.element_affinity 引用（临时元素亲和度变更） |
| `damage-type-def.md` | 元素和伤害类型的 L3 映射表 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 元素属性定义——游戏世界中元素属性的唯一标识。
///
/// ElementDef 只定义元素的身份标识和展示元数据。
/// 元素的克制关系、亲和度加成由 L3 Gameplay 层定义，
/// 因为元素-元素克制关系涉及 ElementId-to-ElementId 引用
/// （L0-to-L0 引用），违反 L0 同层引用禁止规则。
///
/// ElementDef 存在的核心理由是类型安全：系统需要
/// ElementId 而非通用的 TagId 来表达元素相关的运算，
/// 避免将"Humanoid"标签误传入元素伤害计算管线。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ElementDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: ElementId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 表现资源 ──
    /// 图标 Key（用于 UI 中的元素标识）
    pub icon_key: Option<String>,

    /// 元素颜色（十六进制 RGB，用于 UI 着色）
    ///
    /// 元素颜色用于：
    /// - 技能图标中的元素光环
    /// - 伤害数字的颜色
    /// - 元素相关 UI 元素的主题色
    pub color_hex: Option<String>,
}
```

### 字段说明

- **`icon_key`**: 元素图标（如火焰图标、冰晶图标）。在技能描述、Buff 图标、战斗日志中使用
- **`color_hex`**: 元素主题色（如红色代表火焰、蓝色代表冰冻、黄色代表闪电）

### 无引用字段

ElementDef 没有以下常见字段（违反 L0 约束）：

```rust
// 禁止 —— 涉及 ElementId-to-ElementId 引用（L0-to-L0）
// strength_against: Vec<ElementId>,     // 克制目标元素
// weak_against: Vec<ElementId>,          // 被克制元素
// immune_to: Vec<ElementId>,             // 免疫元素

// 禁止 —— 涉及 DamageTypeId 引用（L0-to-L0）
// associated_damage_types: Vec<DamageTypeId>,

// 禁止 —— 涉及 TagId 引用（L0-to-L0）
// tags: Vec<TagId>,
```

这些关系在 L3 Gameplay 层的 `ElementInteractionMatrix` 中定义。

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// 按 ID 查找 ElementDef
pub fn get_element_def(
    id: &ElementId,
    registry: &DefRegistry<ElementDef>,
) -> Option<&ElementDef> {
    registry.get(id)
}
```

### 注册生命周期

```
Load (elements.ron) → Deserialize → Validate → Register (DefRegistry<ElementDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ElementId 不能为空字符串 |
| V2 | `id` 格式合法 | 必须匹配 `^elem:[a-z][a-z0-9_]+$`（如 `elem:fire`、`elem:ice`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 元素必须有显示名称 |
| V5 | `description_key` 非空 | 元素必须有描述文本 |
| V6 | `color_hex` 格式合法（若设置） | 必须匹配 `^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$` |

### 4.2 无跨 Def 引用校验（L0 约束）

ElementDef 是 L0 Def，不引用任何其他 Def。

---

## 5. RON 示例

```ron
// ElementDef 示例 — 基础元素
(
    id: "elem:fire",
    name_key: "elem.fire.name",
    description_key: "elem.fire.desc",
    schema_version: 1,

    icon_key: Some("icons/elements/fire.png"),
    color_hex: Some("#FF4400"),
)
```

```ron
(
    id: "elem:ice",
    name_key: "elem.ice.name",
    description_key: "elem.ice.desc",
    schema_version: 1,

    icon_key: Some("icons/elements/ice.png"),
    color_hex: Some("#44CCFF"),
)
```

```ron
(
    id: "elem:lightning",
    name_key: "elem.lightning.name",
    description_key: "elem.lightning.desc",
    schema_version: 1,

    icon_key: Some("icons/elements/lightning.png"),
    color_hex: Some("#FFCC00"),
)
```

---

## 6. 设计说明

### TagDef 扁平 → ElementDef 出现的原因

如果 TagDef 支持层级结构（parent_id 和继承关系），元素可以直接作为 TagDef 游戏标签表达。但由于 TagDef 保持扁平，需要 ElementDef 提供以下 TagDef 无法提供的能力：

1. **类型安全**：战斗系统通过 `ElementId` 而不是 `TagId` 引用元素，编译期防止类型混淆
2. **与 DamageTypeDef 的正交性**：ElementDef 和 DamageTypeDef 是独立的维度，在 L3 层组合
3. **UI 元数据**：`color_hex` 是元素特有的属性
4. **克制系统的语义锚点**：L3 的 ElementInteractionMatrix 需要 `ElementId` 作为键

### 元素交互矩阵（L3 Gameplay）

元素系统的核心逻辑在 L3 层定义：

```rust
// L3 Gameplay — 元素交互矩阵定义（非 L0，在 L3）
pub struct ElementInteractionMatrix {
    // 每对元素之间的克制倍率
    // elem:fire 对 elem:ice 造成 1.5x 伤害
    pub strengths: Vec<(ElementId, ElementId, f32)>,

    // 每对元素之间的削弱倍率
    // elem:fire 对 elem:fire 造成 0.5x 伤害（同类抗性）
    pub weaknesses: Vec<(ElementId, ElementId, f32)>,

    // 元素-伤害类型映射
    // elem:fire 增强 dmg:fire 和 dmg:burning
    pub damage_type_affinities: Vec<(ElementId, DamageTypeId, f32)>,
}
```

这个矩阵在 L3 加载时校验：
- 所有 ElementId 引用在 DefRegistry<ElementDef> 中存在
- 所有 DamageTypeId 引用在 DefRegistry<DamageTypeDef> 中存在
- 倍率在有效范围内（如 0.0 ~ 3.0）
- 矩阵对称（A 对 B 的克制和 B 对 A 的克制一致性）

### 容纳即可注册，交互矩阵是可选

ElementDef 本身独立注册，不需要元素交互矩阵就能工作。一个只定义了 ElementDef 但不定义 ElementInteractionMatrix 的游戏仍然可以正常运行——只是所有元素的交互倍率为 1.0（无加成/削弱）。

这意味着：
- 小团队可以只注册元素，不定义交互矩阵
- Mod 可以添加新的元素交互矩阵覆盖基础设置
- Mod 可以添加新的 ElementDef 并在自己的交互矩阵中引用

---

*本文档由 @content-architect 维护。*
