---
id: 03-content.definitions.vocabulary.damage-type-def
title: DamageTypeDef — DamageType Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# DamageTypeDef — DamageType Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/domains/combat_domain.md` | **数据 Schema**: `docs/04-data/domains/combat_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

DamageTypeDef 定义了游戏中的**伤害类型**——攻击/技能/效果造成伤害时的分类标签。伤害类型是战斗系统的核心语义单元：

- **Physical 物理系**：Slashing（挥砍）、Piercing（穿刺）、Bludgeoning（钝击）
- **Magical 魔法系**：Fire（火焰）、Ice（冰冻）、Lightning（闪电）、Shadow（暗影）、Holy（神圣）
- **Pure 纯粹系**：True Damage（真实伤害）、%HP Damage（百分比伤害）

### 与其他 L0 Def 的关系

DamageTypeDef 和 TagDef 的区别：TagDef 是通用分类，DamageTypeDef 是**具有伤害语义的具体类型**。一个伤害类型可以映射到对应的 TagDef（如 `DamageTypeDef.id = "dmg:fire"` 对应 `TagDef.id = "tag:fire"`），但两者的职责不同——DamageTypeDef 用于伤害计算管线，TagDef 用于通用分类和条件匹配。

这种"正交映射"模式允许：
- 战斗系统通过 `DamageTypeId` 进行类型安全的伤害计算
- 条件系统通过 `TagId` 进行灵活的标签过滤（如"对火焰类型免疫"）
- 两个体系通过 Content Pipeline 的 Tag-to-DamageType 映射表关联

### 与 ElementDef 的关系

DamageTypeDef 定义伤害的**类型**，ElementDef 定义伤害的**元素属性**。两者是正交关系：

| | 同元素的伤害类型 | 异元素的伤害类型 |
|--|----------------|----------------|
| Fire Damage | `dmg:fire`（物理的火焰伤害？还是魔法的火焰伤害？这取决于 DamageCategory） | — |
| Ice Damage via Fire Skill | 技能造成 `dmg:ice` 类型伤害，但具有 `elem:fire` 元素属性 | 元素和伤害类型不同，用于计算元素加成和伤害类型抗性 |

元素系统在 ElementDef 中定义，与损伤类型系统相互独立。

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `combat_domain.md` | 伤害类型分类、抗性计算规则、伤害公式 |
| `combat_schema.md` | DamageInstance、DamageResistance、Vulnerability 的数据结构 |
| `effect-def.md` | 本 Def 被 EffectDef.damage_type 引用（效果造成的伤害类型） |
| `execution-def.md` | 本 Def 被 ExecutionDef 的伤害公式引用（公式中的伤害类型参数） |
| `element-def.md` | 元素属性和伤害类型的映射关系（L0 层不定义，L3 定义） |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 伤害类型定义——战斗系统中伤害的分类标识。
///
/// DamageTypeDef 定义伤害的"种类"（Fire、Slashing、Pure 等），
/// 不定义伤害的数值表现（那属于 ExecutionDef）。
/// 不定义伤害的元素属性（那属于 ElementDef + L3 映射）。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct DamageTypeDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: DamageTypeId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 伤害分类 ──
    /// 伤害大类：Physical / Magical / Pure
    ///
    /// 分类决定了抗性系统的第一层过滤。
    /// Physical 受护甲影响，Magical 受抗性影响，Pure 无视防御。
    pub damage_category: DamageCategory,

    // ── 表现资源 ──
    /// 图标 Key（用于伤害类型在 UI 中的展示）
    pub icon_key: Option<String>,
}
```

### 内嵌枚举

```rust
/// 伤害大类——决定伤害计算管线中的处理方式
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DamageCategory {
    /// 物理伤害——受目标护甲/防御力影响
    ///
    /// 包括挥砍、穿刺、钝击等。物理伤害计算走"攻击力 vs 防御力"公式。
    /// 可以被 Physical 类别护甲减免。
    Physical,
    /// 魔法伤害——受目标魔法抗性影响
    ///
    /// 包括火焰、冰冻、闪电等。魔法伤害计算走"法术强度 vs 魔法抗性"公式。
    /// 可以被 Magical 类别抗性减免。
    Magical,
    /// 纯粹伤害——无视防御和抗性
    ///
    /// 包括真实伤害、百分比伤害。不经过任何减免计算。
    /// 通常用于特殊技能或环境伤害。
    Pure,
}
```

### 字段说明

- **`damage_category`**: 决定伤害计算管线中的处理分支。Physical 走护甲减伤，Magical 走抗性减伤，Pure 跳过减伤。这是战斗系统在运行时做的第一个分支判断
- **`icon_key`**: 可选的 UI 图标。不同的伤害类型在战斗日志、技能描述中可能需要不同的图标

### 无 tags 字段（L0 约束）

DamageTypeDef 不包含 `tags: Vec<TagId>` 字段。DamageTypeDef 和 TagDef 的关系通过以下方式建立：
1. 内容创作者遵循命名约定（`dmg:fire` 对应 `tag:fire`）
2. 映射表在 Data Schema 层（`!04-data`）定义
3. Content Pipeline 在加载时自动验证映射一致性

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// 按 ID 查找 DamageTypeDef
pub fn get_damage_type_def(
    id: &DamageTypeId,
    registry: &DefRegistry<DamageTypeDef>,
) -> Option<&DamageTypeDef> {
    registry.get(id)
}

/// 按 DamageCategory 过滤
pub fn get_damage_type_defs_by_category(
    category: DamageCategory,
    registry: &DefRegistry<DamageTypeDef>,
) -> Vec<&DamageTypeDef> {
    registry.iter()
        .filter(|def| def.damage_category == category)
        .collect()
}
```

### 注册生命周期

```
Load (damage_types.ron) → Deserialize → Validate → Register (DefRegistry<DamageTypeDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | DamageTypeId 不能为空字符串 |
| V2 | `id` 格式合法 | 必须匹配 `^dmg:[a-z][a-z0-9_]+$`（如 `dmg:fire`、`dmg:slashing`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 伤害类型必须有显示名称 |
| V5 | `description_key` 非空 | 伤害类型必须有描述文本 |
| V6 | `damage_category` 为有效枚举值 | 必须是 DamageCategory 的三个变体之一 |

### 4.2 无跨 Def 引用校验（L0 约束）

DamageTypeDef 是 L0 Def，禁止引用任何其他 Def。因此：
- 无 Tag 引用存在性检查
- 无 Element 引用检查（元素映射在 L3 层定义）
- 无跨层引用检查

### 4.3 命名约定建议

| # | 规则 | 说明 |
|---|------|------|
| V7 | DamageTypeId 和对应 TagId 建议一致 | `dmg:fire` 建议对应 `tag:fire` 标签（Warning 级别） |
| V8 | 同一 `damage_category` 内 ID 建议有前缀 | Physical 类用 `dmg:slashing`、`dmg:piercing`；Magical 类用 `dmg:fire`、`dmg:ice` |

---

## 5. RON 示例

```ron
// DamageTypeDef 示例 — Physical 类别
(
    id: "dmg:slashing",
    name_key: "dmg.slashing.name",
    description_key: "dmg.slashing.desc",
    schema_version: 1,

    damage_category: Physical,
    icon_key: Some("icons/damage_types/slashing.png"),
)
```

```ron
// DamageTypeDef 示例 — Magical 类别
(
    id: "dmg:fire",
    name_key: "dmg.fire.name",
    description_key: "dmg.fire.desc",
    schema_version: 1,

    damage_category: Magical,
    icon_key: Some("icons/damage_types/fire.png"),
)
```

```ron
// DamageTypeDef 示例 — Pure 类别
(
    id: "dmg:true_damage",
    name_key: "dmg.true_damage.name",
    description_key: "dmg.true_damage.desc",
    schema_version: 1,

    damage_category: Pure,
    icon_key: None,
)
```

---

## 6. 设计说明

### 为什么需要 DamageTypeDef 而不是直接用 TagDef？

DamageType 是战斗系统的核心语义概念，具有以下 TagDef 不能满足的需求：

1. **类型安全**：需要 `DamageTypeId` 而不是 `TagId` 以避免战斗管线中的类型错误（如将"Humanoid"标签传入伤害计算）
2. **分类维度**：`DamageCategory` 决定了伤害计算管线的分支——Physical/Magical/Pure 走完全不同的公式
3. **表现元数据**：伤害类型有 `icon_key` 需求，TagDef 不需要

### DamageTypeDef 和 ElementDef 的配合

DamageTypeDef 和 ElementDef 在 L0 层无直接关联。两者的映射在 L3 Gameplay 层定义：

```
L3 Gameplay
  └── Element-DamageType 映射表（Resistance Matrix）
        ├── Fire Element → dmg:fire, dmg:burning（增强）
        ├── Ice Element  → dmg:ice, dmg:cold（增强）
        └── Fire Element → dmg:ice, dmg:cold（削弱）
```

这意味着：
- 一个 `dmg:fire` 通常映射到 `elem:fire` 元素，但不是强制的
- L3 映射系统定义了"火元素增强哪些伤害类型"和"冰元素抵抗哪些伤害类型"
- 这种分离使得 Mod 可以添加新的映射关系而不影响 L0 Def

### 抗性系统

DamageTypeDef 是抗性系统的底层分类。抗性表（Resistance Table）定义在 L3 Gameplay 层：
- 基础抗性：每个 DamageType 对每个实体的默认抗性值
- 类型继承：Physical 类的抗性继承规则（抗 Physical 意味着抗所有 Physical 子类）
- 元素交互：ElementDef 对 DamageTypeDef 的加成/减益

---

*本文档由 @content-architect 维护。*
