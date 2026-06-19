---
id: 03-content.definitions.vocabulary.tag-def
title: TagDef — Tag Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# TagDef — Tag Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/capabilities/tag_domain.md` | **数据 Schema**: `docs/04-data/capabilities/tag_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

TagDef 是整个 Content 体系中最基础的 Def 类型——定义一个**标签标识**，用于实体分类、条件匹配、内容组织。

Tag 是贯穿全系统的**通用分类语言**：
- L1 ConditionDef 通过 TagQuery 检查目标是否持有特定标签
- L1 EffectDef 通过标签过滤目标（"仅对 Humanoid 生效"）
- L2 CharacterDef 通过标签定义角色类型
- L3 QuestDef 通过标签关联任务内容
- L4 MapDef 通过标签标记地图特征

### TagDef 的定位

TagDef 是整个 Content 体系中**唯一没有业务结构字段的 Def 类型**。它的存在仅仅是为了在全球范围内注册一个可引用的标签 ID 及其本地化元数据。标签之间的层级关系（父子、包含、继承）不由 TagDef 本身表达，而是由 Data Schema 层的 `TagHierarchy` 系统构建。

### 与 data Schema 的关系

| 概念 | 归属层 | 说明 |
|------|--------|------|
| `TagId` (tag:fire) | Content (03-content) | Def 的唯一标识符 |
| `TagDef` | Content (03-content) | 内容 Asset：ID + 元数据 + 类别 |
| `TagDefinition` | Data Schema (04-data) | 完整标签定义（含 parent_id、bit_index、namespace） |
| `TagHierarchy` | Data Schema (04-data) | 标签层级树（由 TagDef + 附加配置构建） |
| `TagSet` | Data Schema (04-data) | ECS 组件：实体的标签位掩码 |
| `TagCategory` | Content (03-content) | 标签的三类用途分类 |

TagDef 是 `TagDefinition` 在 Content 层的简化投影——只保留内容创作者关心的字段，将层级、位掩码、命名空间等基础设施细节留给 Data Schema 层。

### TagCategory 三分类

TagDef 引入 `TagCategory` 枚举，将标签按用途分为三类：

| 类别 | 用途 | 示例 | 是否用户可见 | 是否参与 TagHierarchy |
|------|------|------|-------------|---------------------|
| `Gameplay` | 游戏玩法分类 | `tag:fire`, `tag:humanoid`, `tag:healing` | 是 | 是 |
| `Semantic` | 内容管理标签 | `tag:starter_skill`, `tag:elite_monster`, `tag:tutorial_content` | 否 | 否 |
| `System` | 系统内部标签 | `tag:hidden`, `tag:internal`, `tag:deprecated` | 否 | 否 |

**Gameplay 标签**是玩家可见的游戏内分类（元素、种族、职业、伤害类型等），参与层级继承。

**Semantic 标签**是编辑器/工具链使用的管理标签，不参与运行时逻辑（除非显式查询）。典型用途：编辑器过滤（"显示所有标注了 `StarterSkill` 的技能"）、内容审核追踪。

**System 标签**是系统内部使用的特殊标记，通常通过代码而非 RON 注册。典型的 System Tag：`Hidden`（调试用不可见）、`Deprecated`（标记废弃，不参与新内容引用）。

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `tag_domain.md` | 标签层级规则、位掩码实现、TagQuery 匹配模式 |
| `tag_schema.md` | TagDefinition 完整结构（parent_id、bit_index、namespace、is_abstract） |
| `definitions/vocabulary/README.md` | L0 层索引——TagDef 引用规则 |
| 全部 L1-L4 Def 文件 | TagDef 被所有上层 Def 通过 `tags: Vec<TagId>` 引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Tag Def 定义——最基础的标记/分类。
///
/// TagDef 只定义标签的 ID 标识和分类元数据，不包含层级结构。
/// 标签层级关系由 Data Schema 层的 TagHierarchy 系统构建，
/// 后者从 TagDef 注册数据和额外配置中推导层级树。
///
/// TagDef 只定义三层分类（Gameplay/Semantic/System），
/// 不定义命名空间（TagNamespace）- 因为 TagCategory 在内容层面
/// 已经提供了足够的分类维度。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct TagDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: TagId,
    /// 显示名称（本地化 Key）
    ///
    /// Semantic 和 System 类别的标签可能不需要本地化名称，
    /// 但为了一致性仍然提供这个字段。
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key，可选）
    ///
    /// Gameplay 标签通常有描述，Semantic/System 标签可能不需要。
    pub desc_key: Option<LocalizationKey>,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 分类 ──
    /// 标签类别：Gameplay / Semantic / System
    pub category: TagCategory,
}
```

### 内嵌枚举

```rust
/// 标签的三类用途分类
///
/// 分类决定了标签的可见性、参与层级继承的行为、以及工具链的处理方式。
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TagCategory {
    /// 游戏玩法标签——玩家可见，参与层级继承，用于运行时逻辑
    ///
    /// 示例: tag:fire, tag:humanoid, tag:healing
    Gameplay,
    /// 内容管理标签——编辑器/工具链使用，不参与运行时逻辑
    ///
    /// 示例: tag:starter_skill, tag:elite_monster, tag:quest_critical
    Semantic,
    /// 系统内部标签——系统标记用，通常仅代码注册
    ///
    /// 示例: tag:hidden, tag:internal, tag:deprecated
    System,
}
```

### 字段说明

- **`category`**: 标签的三分类设计解决了"一个标签系统服务两种用户（内容创作者和工具链开发者）"的问题。Gameplay 标签是给游戏逻辑使用的，Semantic 标签是给编辑器/工具链使用的，System 标签是给基础设施使用的。三者在一个 Registry 中统一注册，但查询时可按类别过滤
- **`desc_key` 可选**: TagDef 是所有 L0 Def 中唯一 `description_key` 可选的类型。原因是 Semantic/System 类别的标签通常不需要面向用户的描述文本

### 与 data Schema TagDefinition 的关键差异

| 字段 | TagDef (Content) | TagDefinition (Schema) | 理由 |
|------|-----------------|----------------------|------|
| `parent_id` | 无 | 有 | 层级结构由 Schema 层管理，不在 Asset 中表达 |
| `bit_index` | 无 | 有 | 位掩码分配是注册期的自动化过程 |
| `namespace` | 无 | 有 | 命名空间内聚在 Schema 层；Content 层用 TagCategory 替代 |
| `is_abstract` | 无 | 有 | 抽象标签概念属于 Schema 层的层级构建规则 |
| `category` | 有 | 无 | Content 层的三分类是新增的概念，不影响位掩码 |
| `name_key` | 有 | 无 | Schema 层用 path 表达路径，Content 层用独立 name_key |

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// TagDef 通过 L0 批量注册插件加载
///
/// 所有 L0 Def 共用一个 VocabularyPlugin，按固定顺序注册。
pub struct VocabularyPlugin;

impl Plugin for VocabularyPlugin {
    fn build(&self, app: &mut App) {
        // 注册所有 L0 Asset 类型
        app.register_asset::<TagDef>();
        app.register_asset::<AttributeDef>();
        // ... 其他 L0 Def

        // 初始化所有 L0 Registry
        app.insert_resource(DefRegistry::<TagDef>::new());
        app.insert_resource(DefRegistry::<AttributeDef>::new());
        // ... 其他 L0 Registry

        // L0 统一加载系统
        app.add_systems(
            PreUpdate,
            load_vocabulary_defs
                .run_if(resource_changed::<Assets<TagDef>>())
                // ... 所有 L0 Asset 就绪后触发
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按类别过滤 TagDef
pub fn get_tag_defs_by_category(
    category: TagCategory,
    registry: &DefRegistry<TagDef>,
) -> Vec<&TagDef> {
    registry.iter().filter(|def| def.category == category).collect()
}

/// 按 ID 查找 TagDef
pub fn get_tag_def(id: &TagId, registry: &DefRegistry<TagDef>) -> Option<&TagDef> {
    registry.get(id)
}
```

### 注册生命周期

```
Load (tags.ron) → Deserialize → Validate → Register (DefRegistry<TagDef>) → Freeze
    │
    ├── 所有 L0 文件同时加载
    ├── 每个 L0 Def 类型独立反序列化
    ├── 所有 L0 Def 统一校验（无跨 Def 引用校验）
    ├── 各 L0 Def 注入各自 Registry
    └── L0 全部就绪 → L1 开始加载
```

### TagHierarchy 的构建时机

TagHierarchy（标签层级树）由 Data Schema 实现层在以下时机构建：

```
Phase 1: TagDef 全部注册 + Freeze
Phase 2: Data Schema 层读取 TagHierarchy 配置
Phase 3: 从 TagDef + 层级配置构建完整 TagHierarchy（含位掩码分配）
Phase 4: TagHierarchy 校验（无循环、父标签存在、命名空间一致）
Phase 5: L1 开始加载（此时 Tag 查询已可用）
```

TagHierarchy 的构建不属于 Content Pipeline 的职责，但 Content Pipeline 必须保证 L0 全部注册后触发 TagHierarchy 构建事件。

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | TagId 不能为空字符串 |
| V2 | `id` 格式合法 | TagId 必须匹配 `^[a-z][a-z0-9_.-]+$`（如 `tag:fire`, `tag:damage_type.physical`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 即使是 Semantic/System 标签，也必须有本地化 key（可以为空字符串的翻译） |
| V5 | `category` 为有效枚举值 | 必须是 TagCategory 的三个变体之一 |

### 4.2 唯一性校验

| # | 规则 | 说明 |
|---|------|------|
| V6 | TagId 全局唯一 | 所有 tags.ron 文件中不得出现重复的 TagId |
| V7 | TagId 在同一 category 内唯一 | 同一类别下 ID 必须唯一，但不同类别允许同名（技术上禁止，实践中建议也唯一） |

### 4.3 无跨 Def 引用校验

TagDef 是 L0 Def，禁止引用任何其他 Def。因此**没有**以下校验：

- 无 Tag 引用存在性检查（TagDef 本身不引用其他 TagDef）
- 无跨层引用检查（TagDef 不引用任何其他层）
- 无生命周期依赖检查（TagDef 是最先加载的 Def 类型之一）

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | TagId 建议使用层级分隔符 | Gameplay 类标签建议使用 `.` 分隔层级（如 `tag:damage_type.physical.slashing`），以便 TagHierarchy 系统解析 |
| V9 | `desc_key` 使用建议 | Gameplay 标签建议提供描述；Semantic 和 System 标签可选 |

---

## 5. RON 示例

```ron
// TagDef 示例 — Gameplay 类标签
//
// Gameplay 标签是游戏内分类系统的基础。注意：层级关系
// 不在 TagDef 中定义，而是在 TagHierarchy 配置中定义。
(
    id: "tag:fire",
    name_key: "tag.fire.name",
    desc_key: Some("tag.fire.desc"),
    schema_version: 1,
    category: Gameplay,
)
```

```ron
// TagDef 示例 — Semantic 类标签
//
// Semantic 标签用于编辑器/工具链，不需要 desc_key。
(
    id: "tag:starter_skill",
    name_key: "tag.starter_skill.name",
    desc_key: None,
    schema_version: 1,
    category: Semantic,
)
```

```ron
// TagDef 示例 — System 类标签
//
// System 标签用于基础设施标记，没有用户可见信息。
(
    id: "tag:deprecated",
    name_key: "tag.deprecated.name",
    desc_key: None,
    schema_version: 1,
    category: System,
)
```

---

## 6. 设计说明

### 为什么 TagDef 是扁平的？

TagDef 不做层级结构（parent_id、bit_index、namespace）有两层考虑：

1. **Content 职责分离**：TagDef 是内容资产，内容创作者关心的是"有哪些标签可用"，而非"标签在树中的位置"。层级结构是基础设施的优化策略（位掩码查询），不应成为内容创作的负担

2. **Schema 层表达层级**：标签层级由 Data Schema 层的 `TagHierarchy` 系统管理。这允许在未来改变层级实现（从位掩码 u128 升级到 BitSet256）而不影响 TagDef 的 RON 内容

### TagCategory 的设计意图

引入 `TagCategory` 而不是用 TagNamespace 或文件目录表达分类，是因为三分类解决了**实际问题**：Semantic 标签不应该参与游戏逻辑（ConditionDef 不应检查 `tag:starter_skill`），但需要和 Gameplay 标签在同一个 Registry 中统一注册。如果不区分，内容创作者可能无意中在 ConditionDef 中使用 Semantic 标签，导致逻辑错误。

TagCategory 提供了**编译级别的类型安全**——Registry 查询可按类别过滤，Validation 可检测不当使用。

### 标签层级 vs 命名空间

| 概念 | 归属 | 说明 |
|------|------|------|
| TagCategory (Gameplay/Semantic/System) | Content Layer (03-content) | 标签的三类用途——Content 创作者可见 |
| TagNamespace (DamageType/StatusEffect/...) | Data Schema Layer (04-data) | 标签的细分领域——Schema 实现者可见 |
| TagHierarchy (parent/child 树) | Data Schema Layer (04-data) | 标签的层级结构——用于位掩码继承 |

### TagDef 不包含 tags 字段的原因

所有 L1-L3 Def 都有 `tags: Vec<TagId>` 用于分类，但 TagDef 是 L0 Def，禁止引用其他 L0 Def（包括 TagDef 自身）。因此 TagDef 没有 `tags` 字段——它的分类通过 `category` 枚举完成。

---

*本文档由 @content-architect 维护。*
