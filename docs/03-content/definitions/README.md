---
id: 03-content.definitions.README
title: Content Definitions — L0 Vocabulary + L1 Capability + L2 Entity + L3 Gameplay Def 类型索引
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20 (L0 Vocabulary definitions added)
---

# Content Definitions — L0 Vocabulary + L1 Capability + L2 Entity + L3 Gameplay Def 类型索引

> **职责**: @content-architect | **依赖层**: L0 无下层依赖; L1 Capability 依赖 L0 Vocabulary; L2 Entity 依赖 L0-L1 | **被依赖**: L3 Gameplay, L4 World

本文档是 L0 Vocabulary 层、L1 Capability 层、L2 Entity 层和 L3 Gameplay 层所有 Content Def 类型的索引。L1 以下 Def 定义在 `docs/02-domain/capabilities/` 中的领域规则; L2-L3 Def 定义在 `docs/02-domain/domains/` 中。

Content Def 是 Data Schema 的 **Asset 实现层**——每个 Def 类型是一个 Bevy Asset（`#[derive(Asset, TypePath)]`），附带加载、校验、注册的全套管线。

---

## 0. L0 Vocabulary Def 全景

```
L0 Vocabulary Defs                    定义文档
│
├── TagDef               vocabulary/tag-def.md
├── AttributeDef         vocabulary/attribute-def.md
├── DamageTypeDef        vocabulary/damage-type-def.md
├── FactionDef           vocabulary/faction-def.md
├── ElementDef           vocabulary/element-def.md
└── StatusCategoryDef    vocabulary/status-category-def.md
     ↑ 无下层依赖，所有上层 Def 依赖 L0
```

L0 是**基础词汇层**——最小的、不可再分的语义单元。L0 Def 禁止引用任何其他 Def（包括同层 Def）。所有 L0 Def 的自包含设计保证了内容管线可以从最底层开始安全加载。

各 Def 的详细设计见 `vocabulary/` 子目录。L0 层索引见 `vocabulary/README.md`。

---

## 1. L1 Capability Def 全景

```
L1 Capability Defs
│
├── ConditionDef        ← 条件检查（判断是否允许）
├── TriggerDef           ← 触发模式（事件→技能）
├── TargetingDef         ← 目标选择（选谁、怎么选）
├── EffectDef            ← 效果定义（做什么、持续多久）
├── BuffDef              ← 持续状态容器（EffectDef 的持久化封装）
├── ModifierDef          ← 数值修饰模板（修改目标属性）
├── ExecutionDef         ← 执行计算（伤害/治疗公式引用）
├── StackingDef          ← 堆叠规则（多层效果如何叠加）
├── CueDef               ← 表现信号（VFX/SFX/动画）
├── AbilityDef           ← 技能编排（组合上述 Def 为完整能力）
└── SpellDef             ← 法术定义（AbilityDef 的魔法上下文包装）
     ↑ 组合终端，依赖所有其他 L1 Def
```

L1 层采用**模块化组合**设计：一个完整的技能 = AbilityDef 引用 TargetingDef + EffectDef + ConditionDef + CueDef + ExecutionDef + ModifierDef + StackingDef。

---

## 2. 各 Def 类型总览

| # | Def 类型 | 文件 | 领域规则 | 数据 Schema | ID 类型 | 是否可独立注册 |
|---|----------|------|----------|-------------|---------|--------------|
| 1 | `ConditionDef` | `condition-def.md` | `condition_domain.md` | `condition_schema.md` | `ConditionId` | 是 |
| 2 | `TriggerDef` | `trigger-def.md` | `trigger_domain.md` | `trigger_schema.md` | `TriggerId` | 是 |
| 3 | `TargetingDef` | `targeting-def.md` | `targeting_domain.md` | `targeting_schema.md` | `TargetingId` | 是 |
| 4 | `EffectDef` | `effect-def.md` | `effect_domain.md` | `effect_schema.md` | `EffectId` | 是 |
| 5 | `BuffDef` | `buff-def.md` | `effect_domain.md` + `stacking_domain.md` | `effect_schema.md` | `BuffId` | 是 |
| 6 | `ModifierDef` | `modifier-def.md` | `modifier_domain.md` | `modifier_schema.md` | `ModifierId` | 是 |
| 7 | `ExecutionDef` | `execution-def.md` | `execution_domain.md` | `execution_schema.md` | `ExecutionId` | 是 |
| 8 | `StackingDef` | `stacking-def.md` | `stacking_domain.md` | `stacking_schema.md` | `StackingId` | 是 |
| 9 | `CueDef` | `cue-def.md` | `cue_domain.md` | `cue_schema.md` | `CueId` | 是 |
| 10 | `AbilityDef` | `ability-def.md` | `ability_domain.md` | `ability_schema.md` | `AbilityId` | 是（组合终端） |
| 11 | `SpellDef` | `spell-def.md` | `spell_domain.md` | `spell_schema.md` | `SpellId` | 是（AbilityDef 包装） |

---

## 3. 上游依赖（L0 Vocabulary）

所有 L1 Def 依赖以下 L0 Def 类型：

| L0 Def 类型 | 被哪个 L1 Def 引用 | 引用方式 |
|-------------|-------------------|----------|
| `TagDef` | 全部 L1 Def | 标签过滤、分类、条件检查 |
| `AttributeDef` | EffectDef, ModifierDef, ConditionDef, ExecutionDef | 属性修改、属性检查、属性缩放 |
| `DamageTypeDef` | EffectDef, ExecutionDef | 伤害类型标记 |
| `ElementDef` | EffectDef, ExecutionDef, BuffDef | 元素属性标识、元素加成计算 |
| `FactionDef` | ConditionDef | 阵营条件检查（L2/L3 引用为主） |
| `StatusCategoryDef` | BuffDef | 状态类别（增益/减益/控制） |

---

## 4. L2 Entity Defs

L2 Entity 层定义 6 个游戏实体 Def 类型。详细定义见 `entities/` 子目录：

| # | Def 类型 | 文件 | 领域规则 | 数据 Schema | ID 类型 | Base 模式 |
|---|----------|------|----------|-------------|---------|-----------|
| 1 | `CharacterDef` | `entities/character-def.md` | `party_domain.md` | `party_schema.md` | `CharacterId` | CreatureBase |
| 2 | `MonsterDef` | `entities/monster-def.md` | `combat_domain.md` | `combat_schema.md` | `MonsterId` | CreatureBase |
| 3 | `ItemDef` | `entities/item-def.md` | `inventory_domain.md` | `inventory_schema.md` | `ItemId` | ItemBase |
| 4 | `EquipmentDef` | `entities/equipment-def.md` | `inventory_domain.md` | `inventory_schema.md` | `EquipmentId` | ItemBase |
| 5 | `ConsumableDef` | `entities/consumable-def.md` | `inventory_domain.md` | `inventory_schema.md` | `ConsumableId` | ItemBase |
| 6 | `SummonDef` | `entities/summon-def.md` | `summon_domain.md` | `summon_schema.md` | `SummonId` | — |

### L2 的 Base 模式

- **CreatureBase**（生物基座）：CharacterDef 和 MonsterDef 共享的字段集合——基础属性、天生 Ability/Buff/Trigger、阵营、标签。在 Schema 层面内联嵌入两个 Def 中
- **ItemBase**（物品基座）：ItemDef、EquipmentDef 和 ConsumableDef 共享的字段集合——价格、重量、稀有度、图标、标签。在 Schema 层面内联嵌入三个 Def 中

### L2 到 L3 的 Forward Reference

- `CharacterDef.class_id` → `ProgressionDef`（L3 Gameplay）
- `MonsterDef.loot_table` → `LootTableDef`（L3 Gameplay）

这些 Forward Reference 在 L2 加载时被记录但暂不校验，在 L3 加载完成后二次校验。

---

## 5. L3 Gameplay Defs

L3 Gameplay 层定义 8 个玩法系统 Def 类型。详细定义见 `gameplay/` 子目录：

| # | Def 类型 | 文件 | 领域规则 | 数据 Schema | ID 类型 | L4 软引用 |
|---|----------|------|----------|-------------|---------|-----------|
| 1 | `QuestDef` | `gameplay/quest-def.md` | `quest_domain.md` | `quest_schema.md` | `QuestId` | `location_key` |
| 2 | `RecipeDef` | `gameplay/recipe-def.md` | `crafting_domain.md` | `crafting_schema.md` | `RecipeId` | 无 |
| 3 | `ShopDef` | `gameplay/shop-def.md` | `economy_domain.md` | `economy_schema.md` | `ShopId` | 无 |
| 4 | `LootTableDef` | `gameplay/loot-table-def.md` | `economy_domain.md` | `economy_schema.md` | `LootTableId` | 无 |
| 5 | `EncounterDef` | `gameplay/encounter-def.md` | `combat_domain.md` | `combat_schema.md` | `EncounterId` | `position_hint` |
| 6 | `SpawnGroupDef` | `gameplay/spawn-group-def.md` | `combat_domain.md` | `combat_schema.md` | `SpawnGroupId` | 无 |
| 7 | `ProgressionDef` | `gameplay/progression-def.md` | `progression_domain.md` | `progression_schema.md` | `ProgressionId` | 无 |
| 8 | `DifficultyDef` | `gameplay/difficulty-def.md` | `combat_domain.md` | `combat_schema.md` | `DifficultyId` | 无 |

### L3 设计模式

- **Polymorphic Reference 模式**：当需要引用 L2 多种实体类型时（如 LootEntry 可以是 ItemDef/EquipmentDef/ConsumableDef），使用 enum 包装而非多字段
- **Soft Reference 模式**：当需要关联 L4 概念但禁止直接引用时，使用字符串 key 作为松散耦合点（如 QuestDef 的 `location_key`）
- **Forward Reference 接收者**：ProgressionDef 和 LootTableDef 是 L2 CharacterDef/MonsterDef 前向引用的目标，需优先加载并在 L3 完成后二次校验

### L3 到 L2 的引用

L3 Def 广泛引用 L2 Entity Def。关键引用模式：

| Def 类型 | 引用的 L2 Def | 引用方式 |
|----------|--------------|----------|
| QuestDef | CharacterDef, MonsterDef, ItemDef, EquipmentDef, ConsumableDef | 任务目标与奖励 |
| RecipeDef | ItemDef, EquipmentDef, ConsumableDef | 制造材料与产物 |
| ShopDef | ItemDef, EquipmentDef, ConsumableDef | 商品列表 |
| LootTableDef | ItemDef, EquipmentDef, ConsumableDef | 掉落条目 |
| EncounterDef | MonsterDef（间接通过 SpawnGroupDef）, CharacterDef | 胜负条件中的目标 |
| SpawnGroupDef | MonsterDef, EquipmentDef | 怪物条目与装备覆盖 |
| ProgressionDef | — | 不直接引用 L2 |
| DifficultyDef | — | 不直接引用 L2 |

### L3 到 L4 的 Soft Reference

L3 禁止引用 L4，但部分字段需要关联 L4 概念。解决方案是字符串 key 松耦合：

- `QuestDef.location_hint` + `objectives[].location_key` — 位置关联，L4 MapDef 反向映射
- `EncounterDef.groups[].SpawnPosition::Custom(String)` — 位置提示

L4 加载完成后，Content Pipeline 软引用解析验证所有 key 在 L4 侧有定义。

---

## 6. 下游引用者（L2-L4）

### 5.1 L1 Def 被 L2-L4 引用

| 高层层 | Def 类型 | 定义文件 | 引用哪些 L1 Def |
|--------|----------|---------|----------------|
| L2 Entity | `CharacterDef` | `entities/character-def.md` | AbilityDef, EffectDef (常驻), BuffDef (初始状态)，TriggerDef (特质) |
| L2 Entity | `MonsterDef` | `entities/monster-def.md` | AbilityDef, EffectDef, BuffDef, TriggerDef |
| L2 Entity | `EquipmentDef` | `entities/equipment-def.md` | AbilityDef (装备技能)，ModifierDef (装备属性修正)，BuffDef (穿戴 Buff)，ConditionDef (穿戴条件) |
| L2 Entity | `ConsumableDef` | `entities/consumable-def.md` | EffectDef (使用效果)，ConditionDef (使用条件)，TargetingDef (目标规则)，CueDef (使用表现) |
| L2 Entity | `SummonDef` | `entities/summon-def.md` | AbilityDef, EffectDef (召唤/消散效果)，BuffDef (出生 Buff)，ConditionDef (消失条件) |
| L3 Gameplay | `QuestDef` | TBD | ConditionDef (完成条件) |
| L3 Gameplay | `EncounterDef` | TBD | AbilityDef (怪物技能组) |
| L3 Gameplay | `RecipeDef` | TBD | ConditionDef (制造条件) |
| L4 World | `SceneDef` | TBD | ConditionDef (对话分支条件) |
| L4 World | `MapDef` | TBD | ConditionDef (地图解锁条件) |

---

## 7. L1 内部引用关系

L1 Def 之间的引用是**同层引用**，受内容分层规则约束（允许但不得形成循环依赖）：

```
AbilityDef
  ├──→ ConditionDef     (activation_condition, EffectApplication.condition)
  ├──→ TargetingDef     (targeting)
  ├──→ EffectDef        (effects[].effect_def_id)
  ├──→ ExecutionDef     (间接通过 EffectDef)
  ├──→ CueDef           (间接通过 EffectDef)
  ├──→ ModifierDef      (间接通过 EffectDef)
  └──→ StackingDef      (间接通过 EffectDef)

SpellDef
  └──→ AbilityDef       (ability_id——引用 AbilityDef 作为底层能力引擎)
        └──→ (全 L1 依赖链)

EffectDef
  ├──→ ModifierDef      (modifiers[].modifier_def_id, 可选引用)
  ├──→ ConditionDef     (application_condition)
  ├──→ ExecutionDef     (execution.execution_type, 可选)
  ├──→ CueDef           (cues[].cue_def_id)
  └──→ StackingDef      (stacking, 可选引用)

BuffDef
  ├──→ EffectDef        (effect_def_id, 被包装的效果)
  ├──→ ConditionDef     (condition, 激活条件)
  ├──→ StackingDef      (stacking, 可选引用)
  └──→ CueDef           (间接通过 EffectDef)

TriggerDef
  ├──→ ConditionDef     (condition, 额外过滤条件)
  └──→ AbilityDef       (target_ability, 触发时激活)

ExecutionDef → (无 L1 同层依赖, 依赖 L0 AttributeDef + TagDef)

StackingDef → (无 L1 同层依赖, 是纯规则配置)

ConditionDef → (无 L1 同层依赖, 依赖 L0 TagDef + AttributeDef)

TargetingDef → ConditionDef (filter_condition, exclude_condition)

CueDef → ConditionDef (condition, 可选)
```

**禁止的引用模式**（同层循环依赖）：
- EffectDef 不可直接引用 AbilityDef（效果不可引用技能）
- BuffDef 不可引用另一个 BuffDef
- TriggerDef 不可引用另一个 TriggerDef
- 任何 `A → B → A` 的直接或间接循环

---

## 8. 内容资产目录位置

L0 Vocabulary Defs 的 RON 资产位于 `assets/config/00_vocabulary/`：

```
assets/config/00_vocabulary/
├── tags.ron              ← TagDef 集合
├── attributes.ron        ← AttributeDef 集合
├── damage_types.ron      ← DamageTypeDef 集合
├── factions.ron          ← FactionDef 集合
├── elements.ron          ← ElementDef 集合
└── status_categories.ron ← StatusCategoryDef 集合
```

每层目录内的文件遵循**单文件多 Def**原则（详见 `content-layering.md` 8.3 节）。当单个文件超过 2000 行或 50 个 Def 时可拆分为子目录。

L1 Capability Defs 的 RON 资产位于 `assets/config/01_capabilities/`：

```
assets/config/01_capabilities/
├── conditions.ron       ← ConditionDef 集合
├── triggers.ron         ← TriggerDef 集合
├── targeting.ron        ← TargetingDef 集合
├── effects.ron          ← EffectDef 集合
├── buffs.ron            ← BuffDef 集合
├── modifiers.ron        ← ModifierDef 集合
├── executions.ron       ← ExecutionDef 集合
├── stackings.ron        ← StackingDef 集合
├── cues.ron             ← CueDef 集合
└── abilities.ron        ← AbilityDef 集合
```

每层目录内的文件遵循**单文件多 Def**原则（详见 `content-layering.md` 8.3 节）。当单个文件超过 2000 行或 50 个 Def 时可拆分为子目录。

L2 Entity Defs 的 RON 资产位于 `assets/config/02_entities/`：

```
assets/config/02_entities/
├── characters.ron         ← CharacterDef 集合
├── monsters.ron           ← MonsterDef 集合
├── items.ron              ← ItemDef 集合
├── equipment.ron          ← EquipmentDef 集合
├── consumables.ron        ← ConsumableDef 集合
└── summons.ron            ← SummonDef 集合
```

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

---

## 9. 跨文档引用

| 方向 | 文档 | 说明 |
|------|------|------|
| 上游 | `docs/02-domain/capabilities/` | L1 Def 的领域规则——Def 的业务语义定义 |
| 上游 | `docs/02-domain/domains/` | L2-L3 Def 的领域规则（inventory, party, summon, combat + quest, economy, crafting, progression） |
| 上游 | `docs/04-data/capabilities/` | L1 Def 的数据 Schema |
| 上游 | `docs/04-data/domains/` | L2-L3 Def 的数据 Schema |
| 本层 | `definitions/vocabulary/README.md` | L0 Vocabulary Def 类型索引 |
| 本层 | `definitions/vocabulary/tag-def.md` | TagDef 定义（L0） |
| 本层 | `definitions/vocabulary/attribute-def.md` | AttributeDef 定义（L0） |
| 本层 | `definitions/entities/README.md` | L2 Entity Def 类型索引 |
| 本层 | `definitions/gameplay/README.md` | L3 Gameplay Def 类型索引 |
| 本层 | `docs/03-content/content-layering.md` | 5 层分层体系 |
| 本层 | `docs/03-content/content-platform-manifesto.md` | Content Pipeline、Registry、Validation 设计 |
| 下游 | `src/content/` | Content Plugin 实现代码 |
| 下游 | `assets/config/00_vocabulary/` | L0 RON 资产文件 |
| 下游 | `assets/config/01_capabilities/` | L1 RON 资产文件 |
| 下游 | `assets/config/02_entities/` | L2 RON 资产文件 |
| 下游 | `assets/config/03_gameplay/` | L3 RON 资产文件 |
