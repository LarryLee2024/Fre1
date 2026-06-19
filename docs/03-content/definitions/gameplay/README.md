---
id: 03-content.definitions.gameplay.README
title: L3 Gameplay Definitions — Gameplay System Def 类型索引
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# L3 Gameplay Definitions — Gameplay System Def 类型索引

> **Content Layer**: L3 Gameplay | **依赖层**: L0 Vocabulary, L1 Capability, L2 Entity | **被依赖**: L4 World
> **领域规则**: `docs/02-domain/domains/` (quest, economy, crafting, progression, combat) | **数据 Schema**: `docs/04-data/domains/` (quest_schema, economy_schema, crafting_schema, progression_schema)

本文档是 L3 Gameplay 层所有 Content Def 类型的索引。L3 Gameplay 层定义**驱动游戏进程的系统配置**——任务、制造、商店、掉落、遭遇战、生成组、成长曲线、难度配置。这些不是"游戏实体"（那些在 L2），而是"让游戏运转起来的规则系统"。

---

## 1. L3 Gameplay Def 全景

```
L3 Gameplay Defs
│
├── QuestDef              ← 任务系统（目标、前置、奖励、任务链）
├── RecipeDef             ← 制造配方（材料、工艺、产出）
├── ShopDef               ← 商店配置（商品、价格、声望折扣、刷新规则）
├── LootTableDef          ← 掉落表（权重、条件、数量范围、子表）
├── EncounterDef          ← 遭遇战配置（怪物组合、胜负条件、难度覆盖）
├── SpawnGroupDef         ← 怪物生成组（可复用的怪物编队模板）
├── ProgressionDef        ← 职业/等级成长（经验表、职业特性、子职、多职条件）
└── DifficultyDef         ← 难度配置（全局倍率、AI 强度、限制选项）
```

### 设计模式

L3 没有像 L2 CreatureBase/ItemBase 那样的共享字段基座。每个 Def 的字段结构高度差异化，反映各自领域系统的特定需求。L3 引入两种跨 Def 模式：

- **Polymorphic Reference 模式**：当需要引用 L2 多种实体类型时（如 LootTableDef 的条目可以是 ItemDef/EquipmentDef/ConsumableDef），使用 enum 包装而非多字段
- **Soft Reference 模式**：当需要关联 L4 概念（地图位置、场景）但禁止直接引用时，使用字符串 key 作为松散耦合点，由 L4 侧建立反向关联

---

## 2. 各 Def 类型总览

| # | Def 类型 | 文件 | 领域规则 | 数据 Schema | ID 类型 | L4 软引用 |
|---|----------|------|----------|-------------|---------|-----------|
| 1 | `QuestDef` | `gameplay/quest-def.md` | `quest_domain.md` | `quest_schema.md` | `QuestId` | `location_key` (目标地点) |
| 2 | `RecipeDef` | `gameplay/recipe-def.md` | `crafting_domain.md` | `crafting_schema.md` | `RecipeId` | 无 |
| 3 | `ShopDef` | `gameplay/shop-def.md` | `economy_domain.md` | `economy_schema.md` | `ShopId` | 无 |
| 4 | `LootTableDef` | `gameplay/loot-table-def.md` | `economy_domain.md` | `economy_schema.md` | `LootTableId` | 无 |
| 5 | `EncounterDef` | `gameplay/encounter-def.md` | `combat_domain.md` | `combat_schema.md` | `EncounterId` | `position_hint` (地图位置) |
| 6 | `SpawnGroupDef` | `gameplay/spawn-group-def.md` | `combat_domain.md` | `combat_schema.md` | `SpawnGroupId` | 无 |
| 7 | `ProgressionDef` | `gameplay/progression-def.md` | `progression_domain.md` | `progression_schema.md` | `ProgressionId` | 无 |
| 8 | `DifficultyDef` | `gameplay/difficulty-def.md` | `combat_domain.md` | `combat_schema.md` | `DifficultyId` | 无 |

### Forward Reference 双向概览

**L2 -> L3 Forward References**（L2 文档已标注，此处列出以便 L3 管线确保二次校验）：

| L2 Def | 字段 | 引用的 L3 Def | 说明 |
|--------|------|--------------|------|
| CharacterDef | `class_id` | ProgressionDef | L3 加载完成后二次解析 |
| MonsterDef | `loot_table` | LootTableDef | L3 加载完成后二次解析 |

**L3 Soft References -> L4**（非强类型引用，字符串 key 松耦合）：

| L3 Def | 字段 | 说明 |
|--------|------|------|
| QuestDef | `objectives[].location_key` | 到达位置目标，L4 MapDef 或 SceneDef 通过此 key 关联 |
| EncounterDef | `groups[].position_hint` | 生成位置提示（如 "boss_platform"），L4 MapDef 定义精确坐标 |

详情见各 Def 文档的 Forward Reference 小节。

---

## 3. 层间依赖

### 3.1 L3 引用的 L0-L2 Def

| Def 类型 | L0 | L1 | L2 |
|----------|----|----|----|
| `QuestDef` | TagDef, FactionDef | ConditionDef, AbilityDef | CharacterDef, MonsterDef, ItemDef, EquipmentDef, ConsumableDef |
| `RecipeDef` | TagDef | ConditionDef | ItemDef, EquipmentDef, ConsumableDef |
| `ShopDef` | TagDef, FactionDef | ConditionDef | ItemDef, EquipmentDef, ConsumableDef |
| `LootTableDef` | TagDef | ConditionDef | ItemDef, EquipmentDef, ConsumableDef |
| `EncounterDef` | TagDef | ConditionDef, AbilityDef | MonsterDef, CharacterDef |
| `SpawnGroupDef` | TagDef | ConditionDef | MonsterDef, EquipmentDef |
| `ProgressionDef` | TagDef, AttributeDef, FactionDef | ConditionDef, AbilityDef, BuffDef, TriggerDef, ModifierDef | — |
| `DifficultyDef` | TagDef | — | — |

### 3.2 L3 同层引用

| 引用方 | 被引用方 | 用途 |
|--------|---------|------|
| `QuestDef` | `ShopDef`, `RecipeDef`, `QuestDef` | 奖励解锁商店/配方、前置任务链 |
| `RecipeDef` | — | 配方不依赖其他 L3 Def |
| `ShopDef` | — | 商店不依赖其他 L3 Def |
| `LootTableDef` | `LootTableDef` | 子表嵌套（自引用，需避免循环） |
| `EncounterDef` | `SpawnGroupDef`, `DifficultyDef`, `LootTableDef` | 引用生成组、难度覆盖、战利品加成 |
| `SpawnGroupDef` | — | 生成组不依赖其他 L3 Def |
| `ProgressionDef` | — | 成长曲线不依赖其他 L3 Def |
| `DifficultyDef` | — | 难度不依赖其他 L3 Def |

### 3.3 禁止的引用方向

- L3 Def **禁止**引用任何 L4 World Def（MapDef、SceneDef、RegionDef 等）
- `EncounterDef` 不可引用 `MapDef` —— 地图上的遭遇战分配由 L4 MapDef 定义
- `QuestDef` 不可引用 `SceneDef` —— 场景关联由 L4 SceneDef 反向引用 QuestDef
- 违反规则 = 编译模式校验失败

### 3.4 L3 Soft Reference 到 L4

部分 L3 Def 逻辑上需要引用 L4 概念（地图位置、场景 ID）。因层间禁止规则，L3 使用**字符串 key**（非强类型 ID）作为松散耦合点：

```
L3 QuestDef.obj.location_key: "loc:dragon_peak_summit"
                                           ↑
L4 MapDef.waypoints["loc:dragon_peak_summit"] = (45, 120)  ← 从 L4 侧定义映射
```

L4 加载完成后，Content Pipeline 执行**软引用解析**——验证所有 L3 Def 中出现的字符串 key 在 L4 侧有对应定义，并将结果注入运行时查询表。解析失败（key 未定义）产生警告而非错误，允许 L4 后续补全。

此模式的详细设计在 L4 World 层完成时补充。

---

## 4. 资产目录位置

L3 Gameplay Defs 的 RON 资产位于 `assets/config/03_gameplay/`：

```
assets/config/03_gameplay/
├── quests.ron              ← QuestDef 集合
├── recipes.ron             ← RecipeDef 集合
├── shops.ron               ← ShopDef 集合
├── loot_tables.ron         ← LootTableDef 集合
├── encounters.ron          ← EncounterDef 集合
├── spawn_groups.ron        ← SpawnGroupDef 集合
├── progression.ron         ← ProgressionDef 集合
└── difficulties.ron        ← DifficultyDef 集合
```

每层目录内的文件遵循**单文件多 Def**原则（详见 `content-layering.md` 8.3 节）。当单个文件超过 2000 行或 50 个 Def 时可拆分为子目录。

---

## 5. 文档索引

| Def 类型 | 定义文档 | 状态 |
|----------|---------|------|
| QuestDef | `gameplay/quest-def.md` | draft |
| RecipeDef | `gameplay/recipe-def.md` | draft |
| ShopDef | `gameplay/shop-def.md` | draft |
| LootTableDef | `gameplay/loot-table-def.md` | draft |
| EncounterDef | `gameplay/encounter-def.md` | draft |
| SpawnGroupDef | `gameplay/spawn-group-def.md` | draft |
| ProgressionDef | `gameplay/progression-def.md` | draft |
| DifficultyDef | `gameplay/difficulty-def.md` | draft |

---

*本文档由 @content-architect 维护。L3 Gameplay 层的任何变更需通过 Content Architect 审查。*
