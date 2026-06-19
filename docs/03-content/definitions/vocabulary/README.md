---
id: 03-content.definitions.vocabulary.README
title: L0 Vocabulary — 基础词汇层 Def 类型索引
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# L0 Vocabulary — 基础词汇层 Def 类型索引

> **Content Layer**: L0 Vocabulary | **依赖层**: 无（最底层，不可引用任何其他层） | **被依赖**: L1 Capability, L2 Entity, L3 Gameplay, L4 World

本文档是 L0 Vocabulary 层所有 Content Def 类型的索引。L0 定义游戏世界的**基础词汇**——最小的、不可再分的语义单元。所有上层 Def（L1-L4）依赖 L0 作为共同的命名空间和分类体系。

---

## 1. 核心设计原则

### 1.1 L0 职责

- 定义游戏世界的原子概念集合（元素、属性、阵营、伤害类型、状态类别、标签）
- 为所有上层 Def 提供统一的 ID 引用体系
- 提供 Localization 基础键（`name_key`/`description_key`）
- 提供 UI 展示所需的元数据（`icon_key`、`color_hex`）

### 1.2 L0 禁止行为（红线）

- **禁止引用任何其他 Def**（无 L0-to-L0、L0-to-L1+ 引用）
- **禁止同层引用**：TagDef 不可引用 TagDef，AttributeDef 不可引用 TagDef
- **禁止业务逻辑**：L0 Def 不包含条件、公式、行为规则
- **禁止运行时变更**：Def 注册后冻结，不可修改
- **禁止硬编码文本**：所有用户可见文本通过 `LocalizationKey`

### 1.3 L0 词汇量上限

建议不超过 **200 个** L0 Def。词汇量过大意味着分类体系不够抽象，应升维至 L1+。

---

## 2. L0 Def 全景

```
L0 Vocabulary Defs
│
├── TagDef               ← 最基础的标记/分类（Gameplay/Semantic/System 三类）
├── AttributeDef         ← 数值属性定义（Primary/Secondary/Derived/Resource 四类）
├── DamageTypeDef        ← 伤害类型定义（Physical/Magical/Pure 三类）
├── FactionDef           ← 阵营定义（Friendly/Neutral/Hostile 默认态度）
├── ElementDef           ← 元素属性定义（Fire/Ice/Lightning 等类型安全元素 ID）
└── StatusCategoryDef    ← 状态分类定义（用于 Buff 免疫和驱散分类）
```

---

## 3. 各 Def 类型总览

| # | Def 类型 | 文件 | 领域规则 | 数据 Schema | ID 类型 | 是否可独立注册 |
|---|----------|------|----------|-------------|---------|--------------|
| 1 | `TagDef` | `vocabulary/tag-def.md` | `tag_domain.md` | `tag_schema.md` | `TagId` | 是 |
| 2 | `AttributeDef` | `vocabulary/attribute-def.md` | `attribute_domain.md` | `attribute_schema.md` | `AttributeId` | 是 |
| 3 | `DamageTypeDef` | `vocabulary/damage-type-def.md` | `combat_domain.md` | `combat_schema.md` | `DamageTypeId` | 是 |
| 4 | `FactionDef` | `vocabulary/faction-def.md` | `faction_domain.md` | `faction_schema.md` | `FactionId` | 是 |
| 5 | `ElementDef` | `vocabulary/element-def.md` | `combat_domain.md` / `magic_domain.md` | `element_schema.md` | `ElementId` | 是 |
| 6 | `StatusCategoryDef` | `vocabulary/status-category-def.md` | `effect_domain.md` | `status_category_schema.md` | `StatusCategoryId` | 是 |

---

## 4. L0 字段统一约定

所有 L0 Def 遵循统一的字段前缀：

```rust
// ── 统一标识字段 ──
pub id: XxxId,
pub name_key: LocalizationKey,
pub description_key: LocalizationKey,  // TagDef 例外：desc_key: Option<LocalizationKey>
pub schema_version: u32,
```

**例外说明**：TagDef 的 `description_key` 是 `Option<LocalizationKey>`，因为工具链标签（Semantic/System 类别）不需要用户可见描述。

---

## 5. L0 内部字段约束

| 约束 | 说明 | 违反后果 |
|------|------|----------|
| 无 Def 引用字段 | 所有 L0 字段只能是原始类型或枚举 | 编译验证规则触发层间依赖错误 |
| 无 Vec\<TagId\> | L0 Def 不可引用 TagDef | 违反 L0 同层引用禁止规则 |
| 无 Option\<XxxId\> | L0 Def 不可包含任何其他 L0 或更高层的 ID | 加载时验证失败 |
| 枚举字段需完整 | 枚举变体在 Def 定义中一次性确定，不可扩展 | 缺失变体导致游戏逻辑错误 |
| 数值字段有边界 | 所有 f32 字段非 NaN/Infinite | 运行时断言失败 |

---

## 6. 上下游依赖映射

### 6.1 L0 被 L1-L4 引用

| Def 类型 | 被哪个层引用 | 引用方式 |
|----------|-------------|----------|
| `TagDef` | L1-L4 全部层 | `tags: Vec<TagId>`, `filter: TagQuery`, `Category: TagId` |
| `AttributeDef` | L1-L4 全部层 | `base_attributes: Vec<(AttributeId, f32)>`, `ModifierDef.target`, `ConditionDef` 属性检查 |
| `DamageTypeDef` | L1-L4 | `EffectDef.damage_type: DamageTypeId`, `ExecutionDef.damage_type`, `ResistanceModifier` |
| `FactionDef` | L1-L4 | `CharacterDef.faction: FactionId`, `MonsterDef.faction`, `SummonDef.faction_override` |
| `ElementDef` | L1-L4 | `EffectDef.element: ElementId`, `BuffDef.element_affinity`, `ExecutionDef.element_scale` |
| `StatusCategoryDef` | L1-L4 | `BuffDef.category: StatusCategoryId`, `DispelEffect.category_filter`, `ImmunityDef.category` |

### 6.2 L0 不依赖任何层

L0 是独立的原子层。它不导入（`use` 意义上）、不引用（ID 引用意义上）、不依赖任何其他 Content 层。

---

## 7. 内容资产目录位置

L0 Defs 的 RON 资产位于 `assets/config/00_vocabulary/`：

```
assets/config/00_vocabulary/
├── tags.ron              ← TagDef 集合（按 TagCategory 分组存放）
├── attributes.ron        ← AttributeDef 集合
├── damage_types.ron      ← DamageTypeDef 集合
├── factions.ron          ← FactionDef 集合
├── elements.ron          ← ElementDef 集合
└── status_categories.ron ← StatusCategoryDef 集合
```

每个 RON 文件可包含多个 Def，遵循**单文件多 Def**原则（详见 `content-layering.md` 8.3 节）。

---

## 8. 跨文档引用

| 方向 | 文档 | 说明 |
|------|------|------|
| 上游 | `docs/02-domain/capabilities/tag_domain.md` | Tag 领域规则 |
| 上游 | `docs/02-domain/capabilities/attribute_domain.md` | Attribute 领域规则 |
| 上游 | `docs/02-domain/domains/combat_domain.md` | 伤害类型、元素交互规则 |
| 上游 | `docs/02-domain/domains/faction_domain.md` | 阵营规则 |
| 上游 | `docs/02-domain/capabilities/effect_domain.md` | 状态分类规则 |
| 上游 | `docs/04-data/capabilities/tag_schema.md` | Tag 数据 Schema |
| 上游 | `docs/04-data/capabilities/attribute_schema.md` | Attribute 数据 Schema |
| 上游 | `docs/04-data/domains/combat_schema.md` | 伤害类型、元素数据 Schema |
| 上游 | `docs/04-data/domains/faction_schema.md` | 阵营数据 Schema |
| 上游 | `docs/04-data/capabilities/status_category_schema.md` | 状态分类数据 Schema |
| 本层 | `docs/03-content/content-layering.md` | 5 层分层体系 |
| 本层 | `definitions/vocabulary/tag-def.md` | TagDef 定义 |
| 本层 | `definitions/vocabulary/attribute-def.md` | AttributeDef 定义 |
| 本层 | `definitions/vocabulary/damage-type-def.md` | DamageTypeDef 定义 |
| 本层 | `definitions/vocabulary/faction-def.md` | FactionDef 定义 |
| 本层 | `definitions/vocabulary/element-def.md` | ElementDef 定义 |
| 本层 | `definitions/vocabulary/status-category-def.md` | StatusCategoryDef 定义 |
| 下游 | `src/content/plugins/vocabulary_plugin.rs` | L0 批量注册插件 |
| 下游 | `assets/config/00_vocabulary/` | L0 RON 资产目录 |

---

*本文档由 @content-architect 维护。所有 L0 Content Def 变更需经过 Content Architect 审查。*
