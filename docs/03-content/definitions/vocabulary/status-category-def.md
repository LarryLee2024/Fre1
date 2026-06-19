---
id: 03-content.definitions.vocabulary.status-category-def
title: StatusCategoryDef — StatusCategory Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# StatusCategoryDef — StatusCategory Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/capabilities/effect_domain.md`, `docs/02-domain/capabilities/stacking_domain.md` | **数据 Schema**: `docs/04-data/capabilities/status_category_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

StatusCategoryDef 定义了**状态类别**——Buff/Debuff/StatusEffect 的分类标识。状态类别用于三个核心系统：

1. **免疫系统**：生物免疫特定类别的状态（如"亡灵免疫所有物理类 Debuff"）
2. **驱散系统**：驱散技能按类别批量移除状态（如"驱散所有魔法类 Debuff"）
3. **分类过滤**：技能说明中按类别分组显示状态

### 状态类别的两层分类

StatusCategoryDef 提供两层分类维度：

| 维度 | 枚举 | 说明 |
|------|------|------|
| 驱散分组 | `DispelGroup` | 按驱散方式分类：Physical（物理驱散）/ Magical（魔法驱散）/ None（不可驱散） |
| 有益性 | `is_beneficial: bool` | Beneficial（增益）/ Harmful（减益） |

示例分类体系：

```
StatusCategory
├── PhysicalBeneficial    (is_beneficial=true,  DispelGroup=Physical)
├── PhysicalHarmful       (is_beneficial=false, DispelGroup=Physical)
├── MagicalBeneficial     (is_beneficial=true,  DispelGroup=Magical)
├── MagicalHarmful        (is_beneficial=false, DispelGroup=Magical)
├── Control               (is_beneficial=false, DispelGroup=Magical)
├── Unique                (is_beneficial=true,  DispelGroup=None)
└── Permanent             (is_beneficial=true,  DispelGroup=None)
```

### 与 BuffDef 的关系

BuffDef（L1 Capability）通过 `category: StatusCategoryId` 引用 StatusCategoryDef：

```
BuffDef (L1)
  ├── category: StatusCategoryId  → StatusCategoryDef (L0)
  ├── 定义 Buff 的分类归属
  └── 通过分类继承免疫/驱散规则
```

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `effect_domain.md` | Buff 分类规则、免疫系统设计、驱散机制 |
| `stacking_domain.md` | 同类别 Buff 的堆叠规则 |
| `status_category_schema.md` | StatusCategory、DispelGroup 的完整数据结构 |
| `buff-def.md` | 本 Def 被 BuffDef.category 引用 |
| `condition-def.md` | 本 Def 被条件系统中的状态类别检查引用 |
| `effect-def.md` | 本 Def 被驱散类 EffectDef 的类别过滤引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 状态类别定义——Buff/Debuff/StatusEffect 的分类标识。
///
/// StatusCategoryDef 定义了状态属于哪个驱散组别、是增益还是减益。
/// 免疫系统和驱散系统通过 StatusCategoryId 进行批量操作。
///
/// 状态类别是扁平分类（无层级继承）。如果需要"魔法减益"包含
/// "中毒"和"诅咒"，应当分别注册 StatusCategoryDef 然后由
/// BuffDef 各自引用，而非定义层级关系。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct StatusCategoryDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: StatusCategoryId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 分类属性 ──
    /// 驱散分组——此类别状态如何被驱散
    ///
    /// - Physical: 通过物理驱散手段移除（如"净化"、治疗）
    /// - Magical: 通过魔法驱散手段移除（如"驱散魔法"）
    /// - None: 不可驱散（如先天 Buff、永久效果、无来源状态）
    pub dispel_group: DispelGroup,

    /// 是否是有益状态
    ///
    /// true  = 增益（Buff）——在 UI 中以绿色/蓝色显示
    /// false = 减益（Debuff）——在 UI 中以红色/橙色显示
    pub is_beneficial: bool,

    // ── 表现资源 ──
    /// 图标 Key（用于 Buff 栏的默认图标覆盖）
    pub icon_key: Option<String>,
}
```

### 内嵌枚举

```rust
/// 驱散分组——决定此类别状态可被哪种驱散手段移除
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DispelGroup {
    /// 物理驱散——通过"净化"、"治疗"等物理/自然手段驱散
    ///
    /// 物理类的增益/减益通常来自物理环境、普通物品、或非魔法技能。
    /// 例如: 中毒（Posioned）、致盲（Blinded）、战吼（Battle Cry）
    Physical,
    /// 魔法驱散——通过"驱散魔法"、"解除诅咒"等魔法手段驱散
    ///
    /// 魔法类的增益/减益来自法术、魔法物品、或超自然效果。
    /// 例如: 诅咒（Cursed）、魅惑（Charmed）、魔法护盾（Magic Shield）
    Magical,
    /// 不可驱散——无法通过任何常规驱散手段移除
    ///
    /// 特殊类状态只能通过其自身的解除条件移除（等待到期、特定事件触发、死亡）。
    /// 例如: 永久种族 Buff、剧情强制状态、死亡标记
    None,
}
```

### 字段说明

- **`dispel_group`**: 决定驱散系统的行为分支。L1 EffectDef 中的驱散效果通过 `target_categories: Vec<StatusCategoryId>` 指定可驱散的目标类别。当 `DispelGroup::None` 时，该类别下的所有状态免疫所有常规驱散
- **`is_beneficial`**: 决定 UI 显示和行为倾向。增益（true）在 Buff 栏绿色显示，不允许被负面驱散移除；减益（false）红色显示，可被正面驱散移除。免疫系统可据此实现"免疫所有减益"效果
- **`icon_key`**: 状态类别图标在 Buff 栏中作为该类别的默认图标（当 BuffDef 未提供自有图标时使用）

### 无 tags 字段（L0 约束）

StatusCategoryDef 不包含 `tags: Vec<TagId>` 字段，因为 L0 禁止同层引用。

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// 按 ID 查找 StatusCategoryDef
pub fn get_status_category_def(
    id: &StatusCategoryId,
    registry: &DefRegistry<StatusCategoryDef>,
) -> Option<&StatusCategoryDef> {
    registry.get(id)
}

/// 按驱散分组过滤
pub fn get_status_category_defs_by_dispel_group(
    group: DispelGroup,
    registry: &DefRegistry<StatusCategoryDef>,
) -> Vec<&StatusCategoryDef> {
    registry.iter()
        .filter(|def| def.dispel_group == group)
        .collect()
}

/// 获取所有有益/减益类别
pub fn get_status_category_defs_by_beneficial(
    beneficial: bool,
    registry: &DefRegistry<StatusCategoryDef>,
) -> Vec<&StatusCategoryDef> {
    registry.iter()
        .filter(|def| def.is_beneficial == beneficial)
        .collect()
}
```

### 注册生命周期

```
Load (status_categories.ron) → Deserialize → Validate → Register (DefRegistry<StatusCategoryDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | StatusCategoryId 不能为空字符串 |
| V2 | `id` 格式合法 | 必须匹配 `^status:[a-z][a-z0-9_]+$`（如 `status:poison`、`status:bless`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 状态类别必须有显示名称 |
| V5 | `description_key` 非空 | 状态类别必须有描述文本 |
| V6 | `dispel_group` 为有效枚举值 | 必须是 DispelGroup 的三个变体之一 |

### 4.2 无跨 Def 引用校验（L0 约束）

StatusCategoryDef 是 L0 Def，禁止引用任何其他 Def。

---

## 5. RON 示例

```ron
// StatusCategoryDef 示例 — 物理减益（可驱散）
(
    id: "status:physical_harmful",
    name_key: "status.physical_harmful.name",
    description_key: "status.physical_harmful.desc",
    schema_version: 1,

    dispel_group: Physical,
    is_beneficial: false,
    icon_key: Some("icons/status/physical_harmful.png"),
)
```

```ron
// StatusCategoryDef 示例 — 魔法减益（可驱散）
(
    id: "status:magical_harmful",
    name_key: "status.magical_harmful.name",
    description_key: "status.magical_harmful.desc",
    schema_version: 1,

    dispel_group: Magical,
    is_beneficial: false,
    icon_key: Some("icons/status/magical_harmful.png"),
)
```

```ron
// StatusCategoryDef 示例 — 控制效果（魔法驱散）
(
    id: "status:control",
    name_key: "status.control.name",
    description_key: "status.control.desc",
    schema_version: 1,

    dispel_group: Magical,
    is_beneficial: false,
    icon_key: Some("icons/status/control.png"),
)
```

```ron
// StatusCategoryDef 示例 — 不可驱散的增益
(
    id: "status:permanent_buff",
    name_key: "status.permanent_buff.name",
    description_key: "status.permanent_buff.desc",
    schema_version: 1,

    dispel_group: None,
    is_beneficial: true,
    icon_key: Some("icons/status/permanent.png"),
)
```

---

## 6. 设计说明

### 为什么是 L0 而不是 L1？

StatusCategoryDef 被 BuffDef（L1）引用，而 L1 可以引用 L0。如果将 StatusCategoryDef 放在 L1，则：
- BuffDef（L1）引用 StatusCategoryDef（L1）——同层引用，复杂度更高
- 免疫系统（条件检查）需要同时引用 L0 的 TagDef 和 L1 的 StatusCategoryDef——不必要地增加了 L0-L1 的耦合

将 StatusCategoryDef 放在 L0 的核心理由：**分类是基础词汇，不是游戏机制**。`is_beneficial` 和 `dispel_group` 是描述性元数据，不是可执行逻辑。

### 状态类别 vs 标签

为什么用 StatusCategoryDef 而不是 TagDef 来表达状态分类？

| 维度 | StatusCategoryDef | TagDef |
|------|------------------|--------|
| `dispel_group` | 结构化字段 | 用 TagCategory 无法表达 |
| `is_beneficial` | 结构化字段 | 用 TagCategory 无法表达 |
| 免疫查询 | `status_category: StatusCategoryId` | `tags: Vec<TagId>` + TagQuery |
| 语义精确性 | "免疫所有 Physical 类减益" | "免疫所有包含 physical_harmful 标签的 Buff" |
| 运行时性能 | O(1) 分类匹配 | O(n) 标签查询 |

关键差异：状态类别是**有结构的分类**（具有 `dispel_group` 和 `is_beneficial` 两个语义字段），而 TagDef 是**纯标识符**。免疫系统需要按 `dispel_group` 进行批量操作（"对所有 Magical 类减益免疫"），这不是 TagQuery 高效支持的模式。

### 驱散系统的数据结构

```rust
// L1 EffectDef — 驱散效果
pub struct DispelEffectConfig {
    // 目标驱散类别——只驱散匹配这些类别的状态
    pub target_categories: Vec<StatusCategoryId>,

    // 可选：目标驱散分组——按 DispelGroup 批量驱散
    pub target_dispel_groups: Option<Vec<DispelGroup>>,

    // 最多驱散数量
    pub max_dispel_count: u32,

    // 是否区分有益/减益
    pub include_beneficial: bool,
}
```

驱散系统在运行时按以下优先级决定是否驱散一个 Buff：

```
1. BuffDef.category 的 dispel_group 是否匹配驱散效果的 dispel_group?
2. BuffDef.category.is_beneficial 是否符合驱散效果的 include_beneficial?
3. BuffDef.category 是否在驱散效果的 target_categories 列表中?
```

---

*本文档由 @content-architect 维护。*
