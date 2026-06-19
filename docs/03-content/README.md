---
id: 03-content.README
title: Content Architecture — Config Definitions、Registry、Validation
status: draft
owner: content-architect
created: 2026-06-20
tags:
  - content
  - configuration
  - registry
  - validation
  - localization
---

# Content Architecture — 内容架构总纲

> **职责**: @content-architect | **上游输入**: `docs/02-domain/`（领域规则）+ `docs/04-data/`（数据 Schema）
> **下游输出**: `src/content/`（配置加载、校验、注册）

本文档是 Fre 项目内容架构的索引和规范。Content 层横切 Core + Infra，负责将领域规则和数据 Schema 落地为可加载、可校验、可热重载的配置定义。

---

## 1. 架构定位

Content 层在项目中的位置：

```
Domain Rules (02-domain) ──→ Data Schema (04-data) ──→ Content Architecture (03-content)
                                                              │
                                                              ▼
                                              src/content/ (配置加载 + 校验 + 注册)
                                                              │
                                                              ▼
                                              Registry (运行时 Def 访问入口)
```

### 1.1 核心原则

- **Content = Def 的集合** — Content 层只关心 Definition 的定义、加载、校验、注册，不关心 Instance 运行时状态
- **Rule/Content 分离** — 玩法规则下沉 `domain/rules/`，数值配置归 `content/`，禁止配置中出现业务逻辑
- **Definition/Instance 分离** — Content 层输出的 Def 全局不可变，运行时状态独立不写回
- **Single Source Of Truth** — 所有 Definition 通过 Registry 访问，禁止硬编码配置数据

### 1.2 Content Architect 职责

| 职责 | 输出 | 对应目录 |
|------|------|---------|
| Def Schema 设计 | 每个领域的 Def 结构定义 | `docs/03-content/definitions/` |
| Registry 设计 | Def 注册、查找、依赖解析 | `docs/03-content/registry.md` |
| Validation 规则 | Def 校验规则、跨 Def 约束 | `docs/03-content/validation.md` |
| Dependency Graph | Def 间依赖关系、加载顺序 | `docs/03-content/dependency-graph.md` |
| Localization Key | 用户可见文本的 Key 组织规范 | `docs/03-content/localization-keys.md` |
| Asset 目录结构 | RON 文件目录组织 | `docs/03-content/asset-structure.md` |

---

## 2. 目录结构

```
03-content/
├── README.md                     ← 本文件（索引）
├── definitions/                  ← Def Schema 定义
│   ├── ability-def.md            ── AbilityDef Schema
│   ├── effect-def.md             ── EffectDef Schema
│   ├── buff-def.md               ── BuffDef Schema
│   ├── item-def.md               ── ItemDef Schema
│   ├── quest-def.md              ── QuestDef Schema
│   ├── spell-def.md              ── SpellDef Schema
│   ├── terrain-def.md            ── TerrainDef Schema
│   ├── faction-def.md            ── FactionDef Schema
│   ├── progression-def.md        ── ProgressionDef Schema
│   ├── crafting-def.md           ── CraftingDef Schema
│   └── summon-def.md             ── SummonDef Schema
├── registry.md                   ── Registry 架构
├── validation.md                 ── 校验规则
├── dependency-graph.md           ── 依赖图
├── localization-keys.md          ── Localization Key 组织
└── asset-structure.md            ── 资产目录结构
```

---

## 3. Content 类型分类

### 3.1 按加载时机

| 类型 | 加载时机 | 示例 | 可变性 |
|------|---------|------|--------|
| 静态配置 | 启动时 | AbilityDef, EffectDef | 只读 |
| 动态配置 | 运行时 | 可热重载的配置 | 只读但可刷新 |
| 初始数据 | 新游戏时 | 初始存档模板 | 只读 |

### 3.2 按变更频率

| 类型 | 变更频率 | 示例 |
|------|---------|------|
| 核心机制 | 极低 | AttributeDef, TagDef |
| 游戏内容 | 中 | AbilityDef, EffectDef, ItemDef |
| 数值平衡 | 高 | 伤害系数、冷却时间 |
| 本地化 | 中 | Localization 文件 |

---

## 4. Def Schema 设计规范

### 4.1 通用 Def 结构

每个 Def 遵循统一结构：

```rust
#[derive(Asset, TypePath)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub name_key: LocalizationKey,
    pub desc_key: LocalizationKey,
    // 领域特有字段
}
```

### 4.2 Def 命名规范

| 规范 | 规则 |
|------|------|
| 类型名 | `<Domain>Def` 格式，如 `AbilityDef` |
| 文件后缀 | `.ron` |
| ID 字段 | 使用 `id: <Domain>Id` |
| 文本字段 | 使用 `name_key` / `desc_key`（LocalizationKey） |

### 4.3 禁止事项

- 🟥 禁止 Def 包含运行时状态字段
- 🟥 禁止 Def 包含业务逻辑方法
- 🟥 禁止 Def 直接引用另一个 Def 的内部字段（必须通过 ID）
- 🟥 禁止在 Content 层实现游戏逻辑

---

## 5. 与上游文档的映射

| 内容文档 (03-content) | 领域规则 (02-domain) | 数据 Schema (04-data) |
|----------------------|---------------------|---------------------|
| `definitions/ability-def.md` | `capabilities/ability_domain.md` | `capabilities/ability_schema.md` |
| `definitions/effect-def.md` | `capabilities/effect_domain.md` | `capabilities/effect_schema.md` |
| `definitions/buff-def.md` | `capabilities/stacking_domain.md` | `capabilities/stacking_schema.md` |
| `definitions/item-def.md` | `domains/inventory_domain.md` | `domains/inventory_schema.md` |
| `definitions/quest-def.md` | `domains/quest_domain.md` | `domains/quest_schema.md` |
| `definitions/spell-def.md` | `domains/spell_domain.md` | `domains/spell_schema.md` |
| `definitions/terrain-def.md` | `domains/terrain_domain.md` | `domains/terrain_schema.md` |
| `definitions/faction-def.md` | `domains/faction_domain.md` | `domains/faction_schema.md` |
| `definitions/progression-def.md` | `domains/progression_domain.md` | `domains/progression_schema.md` |
| `definitions/crafting-def.md` | `domains/crafting_domain.md` | `domains/crafting_schema.md` |
| `definitions/summon-def.md` | `domains/summon_domain.md` | `domains/summon_schema.md` |

---

## 6. 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | 🟡 draft | content-architect | 2026-06-20 |

---

*本文档由 @content-architect 维护。所有 Def 架构变更需经过 Content Architect 审查。*
