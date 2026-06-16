---
id: domains.README
title: Business Domain Schemas — 业务领域数据架构索引
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - data-architecture
  - index
  - domains
---

# Business Domain Schemas — 业务领域数据架构索引

> **总纲引用**: `docs/04-data/README.md` §8 — 数据目录结构
> **领域规则来源**: `docs/02-domain/`（30 个领域规则文档）

本文档是所有 15 个业务领域 Schema 的索引和快速参考表。

---

## 1. 依赖链概览

```
Foundation Layer (战术空间)
  Tactical ←── Terrain ←── Faction
       │                      │
       ▼                      ▼
Core Layer (战斗核心)
  Combat ←── Spell ←── Reaction
     │
     ├──→ Progression ←── Inventory
     │
     └──→ Party ←── CampRest

Narrative Layer (叙事内容)
  Narrative ←── Quest

Economy Layer (经济系统)
  Economy ←── Crafting ←── Summon
```

---

## 2. 快速参考表

| # | 领域 | 文件 | 层 | Replay | 定义来源 | ID 前缀 |
|---|------|------|-----|--------|---------|---------|
| 1 | Tactical | `tactical_schema.md` | Instance | ✅ | `tactical_domain.md` | — |
| 2 | Terrain | `terrain_schema.md` | Def+Inst+Persist | ✅ | `terrain_domain.md` | `ter_` |
| 3 | Faction | `faction_schema.md` | Def+Inst+Persist | ✅ | `faction_domain.md` | `fct_` |
| 4 | Combat | `combat_schema.md` | Instance+Persist | ✅ | `combat_domain.md` | — |
| 5 | Spell | `spell_schema.md` | Def+Inst+Persist | ✅ | `spell_domain.md` | `spl_` |
| 6 | Reaction | `reaction_schema.md` | Instance | ✅ | `reaction_domain.md` | — |
| 7 | Progression | `progression_schema.md` | Def+Inst+Persist | ❌ | `progression_domain.md` | — |
| 8 | Inventory | `inventory_schema.md` | Def+Inst+Persist | ❌ | `inventory_domain.md` | `itm_` |
| 9 | Party | `party_schema.md` | Inst+Persist | ❌ | `party_domain.md` | `bnd_` |
| 10 | CampRest | `camp_rest_schema.md` | Inst+Persist | ❌ | `camp_rest_domain.md` | `cmp_` |
| 11 | Narrative | `narrative_schema.md` | Def+Inst+Persist | ❌ | `narrative_domain.md` | `dlg_`, `cut_` |
| 12 | Quest | `quest_schema.md` | Def+Inst+Persist | ❌ | `quest_domain.md` | `qst_` |
| 13 | Economy | `economy_schema.md` | Def+Inst+Persist | ❌ | `economy_domain.md` | `shp_` |
| 14 | Crafting | `crafting_schema.md` | Def+Inst+Persist | ❌ | `crafting_domain.md` | `rcp_`, `enc_` |
| 15 | Summon | `summon_schema.md` | Def+Instance | ✅ | `summon_domain.md` | `sum_` |

---

## 3. 各领域 Schema 的 Consitution Check 汇总

每条 Data Law 被违反的 Schema 数量（0 = 全部合规）：

| Data Law | 描述 | 违反数 | 违反领域 |
|----------|------|--------|---------|
| 001 | Def-Instance 强制分离 | 0 | — |
| 002 | Rule-Content 强制分离 | 0 | — |
| 003 | 配置只引用 ID | 0 | — |
| 004 | Ability 不拥有行为 | 0 | — |
| 005 | Effect 是唯一业务执行入口 | 0 | — |
| 006 | Modifier 不拥有业务逻辑 | 0 | — |
| 007 | Duration 属于 Effect | 0 | — |
| 008 | 堆叠行为归属 Stacking | 0 | — |
| 009 | 表现必须经过 Cue | 0 | — |
| 010 | Replay 优先于便利 | 0 | — |
| 011 | Schema 必须版本化 | 0 | — |
| 012 | 域间禁止直接数据引用 | 0 | — |

**所有 15 个业务领域 Schema 完全合规，无 Data Exemption。**

---

## 4. 持久化状态总览

以下领域有独立的 Persistence 层结构，在存档中分别序列化：

```
Save File → DomainStates
  ├── terrain:     Option<TerrainState>        — 被修改格子的表面覆盖
  ├── faction:     FactionState                — 声望、关系变更
  ├── combat:      Option<CombatSnapshot>       — 战斗进行中的快照
  ├── spell:       SpellState                  — 法术位、法术书、专注
  ├── progression: ProgressionState            — 经验、等级、天赋、子职
  ├── inventory:   InventoryState              — 物品实例、装备
  ├── party:       PartyState                  — 成员名册、羁绊、阵型
  ├── camp_rest:   CampRestState               — 生命骰、营地NPC、上次长休
  ├── narrative:   NarrativeState              — StoryFlag、对话历史
  ├── quest:       QuestLog                    — 任务状态、目标进度
  ├── economy:     EconomyState                — 钱包、商店库存
  └── crafting:    CraftingState               — 已解锁配方、制作进度
```

无独立 Persistence 的领域：
- **Tactical**: 位置数据随 Entity 序列化
- **Reaction**: 只有 `ReactionState.used` 随 CombatSnapshot 保存
- **Summon**: 召唤物作为 CombatParticipant 随 CombatSnapshot 保存

---

## 5. 各领域的主要 Definition 结构

| 领域 | Definition 结构 | 内容团队维护 | 热重载 |
|------|----------------|-------------|--------|
| Terrain | `TileProperties` (enum), `HazardZoneDef`, `TerrainInteractionDef` | ✅ | ✅ |
| Faction | `FactionDef`, `FactionRelation` (enum) | ✅ | ✅ |
| Combat | `VictoryConditionDef` | ✅ | ✅ |
| Spell | `SpellDef` (环阶/组件/射程/升环) | ✅ | ✅ |
| Progression | `LevelProgression` (经验表/熟练加值表) | ✅ | ❌（启动时加载） |
| Inventory | `ItemDef`, `LootTableDef` | ✅ | ✅ |
| Party | `BondDef`, `FormationDef` | ✅ | ✅ |
| CampRest | `CampEventDef` | ✅ | ✅ |
| Narrative | `DialogueTreeDef`, `CutsceneDef` | ✅ | ✅ |
| Quest | `QuestDef`, `ObjectiveDef`, `QuestRewardDef` | ✅ | ✅ |
| Economy | `ShopDef` | ✅ | ✅ |
| Crafting | `RecipeDef`, `EnchantmentDef` | ✅ | ✅ |
| Summon | `SummonTemplateDef` | ✅ | ✅ |

---

## 6. 各领域的主要 Instance 结构

| 领域 | Instance 结构 | 瞬时/持久 | ECS Component? |
|------|--------------|-----------|---------------|
| Tactical | `GridPosition`, `MovementPoints`, `Facing` | Persistent | ✅ |
| Terrain | `SurfaceOverride`, `TerrainAttachEffect` | Persistent | ✅ |
| Faction | `FactionMembership`, `Reputation`, `RelationshipState` | 持久/瞬时混合 | ✅ |
| Combat | `CombatState`, `TurnOrder`, `CombatParticipant`, `ActionPoints` | Persistent | ✅ |
| Combat | `CombatIntent`, `DamageResult` | **瞬时** | ❌ |
| Spell | `SpellSlotPool`, `Spellbook`, `Concentration` | Persistent | ✅ |
| Reaction | `ReactionState` | Persistent | ✅ |
| Reaction | `ReactionQueue`, `OpportunityAttackData`, `CounterspellData` | **瞬时** | ❌ |
| Progression | `Experience`, `LevelComponent`, `TalentTree`, `SubclassChoice` | Persistent | ✅ |
| Progression | `ASIState` | **瞬时** | ❌ |
| Inventory | `ItemInstance`, `Inventory`, `EquipmentSlots` | Persistent | ✅ |
| Party | `PartyRoster`, `BondState` | Persistent | ✅ |
| CampRest | `RestState`, `HitDicePool`, `CampNPC` | Persistent | ✅ |
| Narrative | `StoryFlag` | Persistent | ✅ |
| Narrative | `DialogueState` | **瞬时** | ❌ |
| Quest | `QuestLog`, `ObjectiveProgress` | Persistent | ✅ |
| Economy | `Wallet`, `ShopInstance`, `Price` | 持久/瞬时混合 | ✅ |
| Crafting | `EnchantmentSlot`, `UpgradeLevel` | Persistent | ✅ |
| Summon | `SummonBond`, `SummonSlotManager` | Persistent | ✅ |

---

## 7. 与 docs/02-domain/ 的对照关系

每个 `docs/04-data/domains/*_schema.md` 文件对应一个 `docs/02-domain/*_domain.md` 文件：

| 数据架构文件 | 领域规则文件 | 状态 |
|-------------|------------|------|
| `tactical_schema.md` | `tactical_domain.md` | ✅ 对齐 |
| `terrain_schema.md` | `terrain_domain.md` | ✅ 对齐 |
| `faction_schema.md` | `faction_domain.md` | ✅ 对齐 |
| `combat_schema.md` | `combat_domain.md` | ✅ 对齐 |
| `spell_schema.md` | `spell_domain.md` | ✅ 对齐 |
| `reaction_schema.md` | `reaction_domain.md` | ✅ 对齐 |
| `progression_schema.md` | `progression_domain.md` | ✅ 对齐 |
| `inventory_schema.md` | `inventory_domain.md` | ✅ 对齐 |
| `party_schema.md` | `party_domain.md` | ✅ 对齐 |
| `camp_rest_schema.md` | `camp_rest_domain.md` | ✅ 对齐 |
| `narrative_schema.md` | `narrative_domain.md` | ✅ 对齐 |
| `quest_schema.md` | `quest_domain.md` | ✅ 对齐 |
| `economy_schema.md` | `economy_domain.md` | ✅ 对齐 |
| `crafting_schema.md` | `crafting_domain.md` | ✅ 对齐 |
| `summon_schema.md` | `summon_domain.md` | ✅ 对齐 |

---

## 8. 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `tactical_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `terrain_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `faction_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `combat_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `spell_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `reaction_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `progression_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `inventory_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `party_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `camp_rest_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `narrative_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `quest_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `economy_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `crafting_schema.md` | ✅ stable | data-architect | 2026-06-16 |
| `summon_schema.md` | ✅ stable | data-architect | 2026-06-16 |

---

## 9. 对下游的提示

### 对 @architect 的交接触发

15 个业务 Schema 全部完成，可据此进行模块边界设计：

- **战术空间层** (Tactical/Terrain/Faction) → `core/domains/tactical/`, `core/domains/terrain/`, `core/domains/faction/`
- **战斗核心层** (Combat/Spell/Reaction) → `core/domains/combat/`, `core/domains/spell/`, `core/domains/reaction/`
- **成长养成层** (Progression/Inventory/Party/CampRest) → `core/domains/progression/`, `core/domains/inventory/`, `core/domains/party/`, `core/domains/camp_rest/`
- **叙事内容层** (Narrative/Quest) → `core/domains/narrative/`, `core/domains/quest/`
- **经济系统层** (Economy/Crafting/Summon) → `core/domains/economy/`, `core/domains/crafting/`, `core/domains/summon/`

### 对 @feature-developer 的提示

每个 Schema 的 Rust struct 定义是实现的起点，但有以下注意事项：

1. **字段名和类型**是建议性的，实现时可根据 ECS 模式调整（如使用 `Component` trait）
2. **四层分离**必须严格遵守：Definition/Spec/Instance/Persistence 不得跨层污染
3. **Persistence 层**的序列化 trait 实现需要 Save 系统提供
4. **瞬时结构**（标记为"瞬时"的）不实现 `Persistable` trait
