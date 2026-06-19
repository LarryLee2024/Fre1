---
id: 03-content.definitions.entities.README
title: L2 Entity Definitions — Game Entity Def 类型索引
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# L2 Entity Definitions — Game Entity Def 类型索引

> **Content Layer**: L2 Entity | **依赖层**: L0 Vocabulary, L1 Capability | **被依赖**: L3 Gameplay, L4 World
> **领域规则**: `docs/02-domain/domains/` (inventory, party, summon) | **数据 Schema**: `docs/04-data/domains/` (inventory_schema, party_schema, summon_schema)

本文档是 L2 Entity 层所有 Content Def 类型的索引。L2 Entity 层定义游戏的**具体实体模板**——角色、怪物、物品、装备、消耗品、召唤物——它们是玩家在游戏中实际交互的对象。

---

## 1. L2 Entity Def 全景

```
L2 Entity Defs
│
├── CharacterDef          ← 角色模板（可操作/可加入队伍）
├── MonsterDef            ← 怪物模板（战斗中的敌人/中立生物）
├── ItemDef               ← 基础物品模板（材料/任务物品/通用物品）
├── EquipmentDef          ← 装备模板（武器/护甲/饰品）
├── ConsumableDef         ← 消耗品模板（药水/卷轴/食物）
└── SummonDef             ← 召唤物模板（临时实体）
```

### 共享设计模式

L2 Entity 层引入两种**基座模式**（Base Pattern）来减少字段重复：

- **`CreatureBase`**（生物基座）—— CharacterDef 和 MonsterDef 共享的基础字段集合，包括基础属性、天生的 Ability/Buff、阵营、标签等
- **`ItemBase`**（物品基座）—— ItemDef、EquipmentDef 和 ConsumableDef 共享的基础字段集合，包括价格、重量、稀有度、图标、标签等

两种基座都是**Schema 级别的内联模式**——每个 Def 类型在自己的 Rust 结构中嵌入这些字段，而不是通过引用关联。这保证了每个 Def 资产是自包含的（self-contained）。

---

## 2. 各 Def 类型总览

| # | Def 类型 | 文件 | 领域规则 | 数据 Schema | ID 类型 | 基座 |
|---|----------|------|----------|-------------|---------|------|
| 1 | `CharacterDef` | `character-def.md` | `party_domain.md` | `party_schema.md` | `CharacterId` | CreatureBase |
| 2 | `MonsterDef` | `monster-def.md` | `combat_domain.md` | `combat_schema.md` | `MonsterId` | CreatureBase |
| 3 | `ItemDef` | `item-def.md` | `inventory_domain.md` | `inventory_schema.md` | `ItemId` | ItemBase |
| 4 | `EquipmentDef` | `equipment-def.md` | `inventory_domain.md` | `inventory_schema.md` | `EquipmentId` | ItemBase |
| 5 | `ConsumableDef` | `consumable-def.md` | `inventory_domain.md` | `inventory_schema.md` | `ConsumableId` | ItemBase |
| 6 | `SummonDef` | `summon-def.md` | `summon_domain.md` | `summon_schema.md` | `SummonId` | — |

---

## 3. 层间依赖

### 3.1 L2 引用的 L0-L1 Def

L2 Entity 层依赖 L0 Vocabulary 和 L1 Capability：

| Def 类型 | 引用的 L0 Def | 引用的 L1 Def |
|----------|--------------|--------------|
| `CharacterDef` | TagDef, AttributeDef, FactionDef | AbilityDef, BuffDef, EffectDef, TriggerDef |
| `MonsterDef` | TagDef, AttributeDef, FactionDef | AbilityDef, BuffDef, EffectDef |
| `ItemDef` | TagDef | — |
| `EquipmentDef` | TagDef, AttributeDef | ModifierDef, AbilityDef, BuffDef |
| `ConsumableDef` | TagDef | EffectDef, ConditionDef, TargetingDef |
| `SummonDef` | TagDef, AttributeDef, FactionDef | AbilityDef, BuffDef, EffectDef, ConditionDef |

### 3.2 L2 到 L3 的 Forward Reference（前向引用）

以下 L2 Def 包含对 L3 Gameplay 层的引用。由于 L3 尚未定义，这些引用标注为 **Forward Reference**：

- `CharacterDef.class_id` → `ProgressionDef` (L3)
- `MonsterDef.loot_table` → `LootTableDef` (L3)

**处理规则**：
- 加载时：L2 加载管线**不应**因 Forward Reference 解析失败而报错
- 校验时：Forward Reference 的校验推迟到 L3 加载完成后二次校验
- 运行时访问：通过 `LazyRef<T>` 包装，在 L3 Registry 可用时才解析

### 3.3 禁止的引用方向

- L2 Def **禁止**引用 L3 Gameplay Def（除上述标注的 Forward Reference）
- L2 Def **禁止**引用 L4 World Def
- 违反规则 = 编译模式校验失败

---

## 4. 资产目录位置

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

每层目录内的文件遵循**单文件多 Def**原则。当单个文件超过 2000 行或 50 个 Def 时可拆分为子目录。

---

## 5. 文档索引

| Def 类型 | 定义文档 | 状态 |
|----------|---------|------|
| CharacterDef | `entities/character-def.md` | draft |
| MonsterDef | `entities/monster-def.md` | draft |
| ItemDef | `entities/item-def.md` | draft |
| EquipmentDef | `entities/equipment-def.md` | draft |
| ConsumableDef | `entities/consumable-def.md` | draft |
| SummonDef | `entities/summon-def.md` | draft |

---

*本文档由 @content-architect 维护。L2 Entity 层的任何变更需通过 Content Architect 审查。*
