---
id: 02-domain.README
title: Domain Rules — 领域规则索引
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - index
  - governance
---

# Domain Rules — 领域规则索引

> **架构依据**: `docs/01-architecture/README.md` — DDD三层+横切四层架构总纲
> **数据映射**: `docs/04-data/README.md` — 数据架构总纲

本文档是 `docs/02-domain/` 下全部 30 个领域规则文件的索引和快速参考。

---

## 1. 领域分类

30 个领域分为三大类：**Capabilities**（能力系统，15 个）、**Infrastructure**（基础设施，3 个）、**Business Domains**（业务领域，15 个）。

### 1.1 Capabilities — 能力系统（15）

能力领域构成游戏的核心机制骨架，从底层数据结构到上层行为表现：

```
基础数据层
  Tag ──────→ Attribute
    │            │
    ├──→ Modifier ──→ Aggregator
    │
    └──→ GameplayContext

配置/条件层
  Spec ────→ Condition ────→ Trigger ────→ Event

行为表现层
  Ability ──→ Targeting ──→ Execution ──→ Effect
                                             │
                                      ┌──────┼──────┐
                                      ▼      ▼      ▼
                                   Stacking Cue
```

| # | 文件 | 层 | 职责摘要 |
|---|------|-----|---------|
| 01 | `tag_domain.md` | 基础数据 | 标签体系——最底层的标记/分类机制 |
| 02 | `attribute_domain.md` | 基础数据 | 属性体系——角色数值属性的定义与计算 |
| 03 | `modifier_domain.md` | 基础数据 | Modifier——数值修改的原子操作 |
| 04 | `aggregator_domain.md` | 基础数据 | 聚合器——Modifier 的叠加与聚合计算 |
| 05 | `gameplay_context_domain.md` | 基础数据 | 玩法上下文——战斗中影响数值计算的瞬时环境 |
| 06 | `spec_domain.md` | 配置/条件 | Spec——Definition → Instance 的桥梁层 |
| 07 | `condition_domain.md` | 配置/条件 | 条件系统——前置条件、触发条件、过滤条件 |
| 08 | `trigger_domain.md` | 配置/条件 | 触发器——事件驱动的条件响应机制 |
| 09 | `event_domain.md` | 配置/条件 | 事件系统——领域间通信的基础设施 |
| 10 | `ability_domain.md` | 行为表现 | 技能——角色可执行的行动模板 |
| 11 | `targeting_domain.md` | 行为表现 | 目标选择——技能目标的选取规则与校验 |
| 12 | `execution_domain.md` | 行为表现 | 执行——伤害/治疗的数值结算 |
| 13 | `effect_domain.md` | 行为表现 | 效果——所有"结果"的载体（伤害/Buff/地形变化） |
| 14 | `stacking_domain.md` | 行为表现 | 堆叠——同类型效果的叠加规则 |
| 15 | `cue_domain.md` | 行为表现 | 表现信号——Effect → VFX/SFX/UI 的桥梁 |

### 1.2 Business Domains — 业务领域（15）

业务领域实现具体的游戏功能，分层依赖 Capabilities：

```
Foundation Layer (战术空间)
  tactical_domain.md ←── terrain_domain.md ←── faction_domain.md
       │                         │
       ▼                         ▼
Core Layer (战斗核心)
  combat_domain.md ←── spell_domain.md ←── reaction_domain.md
       │
       ├──→ progression_domain.md ←── inventory_domain.md
       │
       └──→ party_domain.md ←── camp_rest_domain.md

Narrative Layer (叙事内容)
  narrative_domain.md ←── quest_domain.md

Economy Layer (经济系统)
  economy_domain.md ←── crafting_domain.md ←── summon_domain.md
```

| # | 文件 | 层 | 职责摘要 |
|---|------|-----|---------|
| 16 | `tactical_domain.md` | Foundation | 战术空间——网格位置、移动、掩体、夹击 |
| 17 | `terrain_domain.md` | Foundation | 地形——Tile、表面类型、陷阱、通行性 |
| 18 | `faction_domain.md` | Foundation | 阵营关系——阵营定义、声望、关系判定 |
| 19 | `combat_domain.md` | Core | 战斗——回合流程、先攻、伤害结算、胜负 |
| 20 | `spell_domain.md` | Core | 法术——法术位、专注、豁免、升环 |
| 21 | `reaction_domain.md` | Core | 反应——机会攻击、法术反制、护盾、援护 |
| 22 | `progression_domain.md` | Core | 成长——经验、等级、职业、天赋、ASI |
| 23 | `inventory_domain.md` | Core | 背包——物品、装备槽位、消耗品、战利品 |
| 24 | `party_domain.md` | Core | 队伍——成员名册、羁绊、阵型、换人 |
| 25 | `camp_rest_domain.md` | Core | 营地/休息——短休、长休、生命骰、营地事件 |
| 26 | `narrative_domain.md` | Narrative | 叙事——对话树、StoryFlag、演出 |
| 27 | `quest_domain.md` | Narrative | 任务——目标追踪、奖励、前置条件 |
| 28 | `economy_domain.md` | Economy | 经济——货币、商店、价格、交易 |
| 29 | `crafting_domain.md` | Economy | 制作——配方、附魔、装备升级 |
| 30 | `summon_domain.md` | Economy | 召唤——召唤物模板、专注绑定、消失 |

---

## 2. 依赖关系总图

### 2.1 Capabilities 间依赖

```
Tag ──────→ Condition ──────→ Trigger ───→ Event
  │                              │
  ├──→ Modifier ──→ Aggregator   │
  │                    │         │
  │                    ▼         ▼
  │              Attribute    Ability
  │                    │         │
  │                    │         ▼
  │                    └──→ Targeting ←── GameplayContext
  │                              │
  │                              ▼
  │                         Execution
  │                              │
  │                              ▼
  └────────────────────────→ Effect ←── Spec
                                    │
                             ┌──────┼──────┐
                             ▼      ▼      ▼
                          Stacking Cue   Event
```

### 2.2 业务领域间依赖

```
Foundation:
  Tactical ←── Terrain ←── Faction

Core:
  Combat ←── Spell ←── Reaction
    │
    ├──→ Progression ←── Inventory
    │
    └──→ Party ←── CampRest

Narrative:
  Narrative ←── Quest

Economy:
  Economy ←── Crafting ←── Summon
```

### 2.3 Capabilities → 业务领域引用

```
Capabilities 被 Business Domains 引用的关系：

Tag           → Terrain (SurfaceType), Faction (BondRequirement), Quest (ObjectiveType)
Attribute     → Progression (ASI), Spell (SaveDC), Combat (Initiative)
Modifier      → Inventory (Equipment), Progression (Talent), Party (Bond)
Condition     → Quest (Prerequisite), Narrative (ChoiceCondition), Inventory (EquipCondition)
Ability       → Spell (复用生命周期), Summon (召唤能力), Combat (行动)
Effect        → Terrain (TerrainEffect), Spell (法术效果), Inventory (消耗品)
Event         → 所有领域（领域间通信）
```

---

## 3. 各文档结构标准

每个领域规则文档遵循 8 节标准结构：

```
1. 统一术语       — 术语表（术语、定义、职责边界）
2. 状态机         — 核心状态流转图（如有）
3. 不变量         — 领域不变量（条件 + 不变规则 + 违反后果）
4. 禁止事项       — 明确禁止的行为及理由
5. 流程定义       — 输入 → 处理 → 输出 → 失败处理
6. 领域事件       — 事件表 + 订阅关系图
7. 对齐校验       — 与已有架构的对齐检查
8. 自检清单       — 8 项质量检查项
```

---

## 4. 与 docs/04-data/ 的映射

每个领域规则文档对应一个数据架构 Schema 文档：

| 领域规则 (02-domain) | 数据架构 (04-data) |
|---------------------|-------------------|
| `tag_domain.md` | `capabilities/tag_schema.md` |
| `attribute_domain.md` | `capabilities/attribute_schema.md` |
| `modifier_domain.md` | `capabilities/modifier_schema.md` |
| `aggregator_domain.md` | `capabilities/aggregator_schema.md` |
| `gameplay_context_domain.md` | `capabilities/gameplay_context_schema.md` |
| `spec_domain.md` | `capabilities/spec_schema.md` |
| `condition_domain.md` | `capabilities/condition_schema.md` |
| `trigger_domain.md` | `capabilities/trigger_schema.md` |
| `ability_domain.md` | `capabilities/ability_schema.md` |
| `targeting_domain.md` | `capabilities/targeting_schema.md` |
| `execution_domain.md` | `capabilities/execution_schema.md` |
| `effect_domain.md` | `capabilities/effect_schema.md` |
| `stacking_domain.md` | `capabilities/stacking_schema.md` |
| `event_domain.md` | `capabilities/event_schema.md` |
| `cue_domain.md` | `capabilities/cue_schema.md` |
| `tactical_domain.md` | `domains/tactical_schema.md` |
| `terrain_domain.md` | `domains/terrain_schema.md` |
| `faction_domain.md` | `domains/faction_schema.md` |
| `combat_domain.md` | `domains/combat_schema.md` |
| `spell_domain.md` | `domains/spell_schema.md` |
| `reaction_domain.md` | `domains/reaction_schema.md` |
| `progression_domain.md` | `domains/progression_schema.md` |
| `inventory_domain.md` | `domains/inventory_schema.md` |
| `party_domain.md` | `domains/party_schema.md` |
| `camp_rest_domain.md` | `domains/camp_rest_schema.md` |
| `narrative_domain.md` | `domains/narrative_schema.md` |
| `quest_domain.md` | `domains/quest_schema.md` |
| `economy_domain.md` | `domains/economy_schema.md` |
| `crafting_domain.md` | `domains/crafting_schema.md` |
| `summon_domain.md` | `domains/summon_schema.md` |

---

## 5. 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | ✅ stable | domain-designer | 2026-06-16 |
| `tag_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `attribute_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `modifier_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `aggregator_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `gameplay_context_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `spec_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `condition_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `trigger_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `ability_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `targeting_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `execution_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `effect_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `stacking_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `event_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `cue_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `tactical_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `terrain_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `faction_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `combat_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `spell_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `reaction_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `progression_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `inventory_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `party_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `camp_rest_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `narrative_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `quest_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `economy_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `crafting_domain.md` | ✅ stable | domain-designer | 2026-06-16 |
| `summon_domain.md` | ✅ stable | domain-designer | 2026-06-16 |

---

## 6. 角色分工

| 角色 | 对本目录的职责 |
|------|-------------|
| **@domain-designer** | 定义"规则是什么"——维护领域术语、不变量、流程定义 |
| **@data-architect** | 定义"规则如何表达"——将领域规则映射为 Schema |
| **@feature-developer** | 实现——将领域规则转为 Rust 代码 |
| **@test-guardian** | 验证——确保实现符合领域不变量 |
