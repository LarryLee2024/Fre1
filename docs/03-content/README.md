---
id: 03-content.README
title: Content Platform Architecture — 5-Layer Content Platform (L0-L3 complete)
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20 (L3 Gameplay definitions added)
tags:
  - content
  - content-platform
  - 5-layer
  - registry
  - validation
  - governance
---

# Content Platform Architecture — 内容平台架构总纲

> **职责**: @content-architect | **上游输入**: `docs/02-domain/`（领域规则）+ `docs/04-data/`（数据 Schema）
> **下游输出**: `src/content/`（配置加载、校验、注册）+ `assets/config/`（RON 资产目录结构）

本文档定义 Fre 项目 Content Platform 的整体架构。Content Platform 不是"配置文件加载器"，而是**游戏数据库引擎**——从 Schema 设计、资产加载、校验、注册到运行时查询的全链路基础设施。

---

## 1. 架构定位

### 1.1 Content 层在项目中的位置

Content 层是横切层（Cross-cutting Layer），横跨 Core + Infra：

```
Domain Rules (02-domain) ──→ Data Schema (04-data) ──→ Content Platform (03-content)
                                                               │
                     ┌─────────────────────────────────────────┼─────────────────────────┐
                     ▼                                         ▼                         ▼
              assets/config/                             src/content/              Mod Content
          (RON 资产 — 5层目录)                     (Content Pipeline 实现)       (Mod RON 覆盖/扩展)
                     │                                         │                         │
                     └─────────────────────────────────────────┼─────────────────────────┘
                                                               ▼
                                               Registry (运行时 Def 访问入口)
                                                               │
                                                               ▼
                                          Core Domains (加载后的业务逻辑消费)
```

### 1.2 与项目架构的映射

| 项目横切层 | Content Platform 中的角色 | 责任 |
|-----------|--------------------------|------|
| `shared/` | 内容类型 ID 系统 (`*Id`), 通用验证工具 | 无业务语义的 ID 生成与校验 |
| `core/` | Def Schema (Rust struct), Domain 规则消费 | Schema 定义 + Def 消费 |
| `infra/` | Registry, Pipeline, 存档 | 注册、查询、持久化 |
| `content/` | Plugin, Loader, Validator | 加载->反序列化->校验->注册->冻结 |
| `modding/` | Mod Content Provider | Mod 内容注册、冲突检测、版本兼容 |

### 1.3 内容平台核心原则

- **Content = 游戏数据库** — 每个 RON 文件是一条数据库记录，不是配置文件。Content Platform 是数据库引擎，不是配置加载器。
- **5层分层（L0-L4）** — 内容按抽象层级分为 5 层，禁止高层引用低层（详见 `content-layering.md`）。
- **Load → Deserialize → Validate → Register → Freeze** — 严格管线顺序，配置错误在加载时捕获，不在运行时捕获。
- **Registry 是唯一入口** — 所有 Def 访问通过 `DefRegistry<T>`，禁止直接读 RON 文件或 hardcode 字符串。
- **Definition/Instance 分离** — Content 层只产出全局不可变的 Def。运行时状态独立不写回。
- **ID 唯一性** — 每个 Def 有全局唯一的 `id` 字段（强类型 ID），禁止通过文件名或路径标识内容。
- **Localization First** — 所有用户可见文本字段使用 `LocalizationKey`，禁止硬编码字符串。
- **Mod 扩展优先** — 架构必须支持 Mod 通过同一管线注册新 Def，无需修改核心代码。

---

## 2. 5-Layer 内容层级总览

Content Platform 将游戏内容按抽象层级分为 5 层：

```
L4:      World（叙事世界层）
         MapDef / RegionDef / SceneDef / CutsceneDef
         NarrativeArcDef / StoryFlagDef / CompanionDef
         ↑ 依赖 L0-L3

L3:      Gameplay（玩法系统层）
         QuestDef / RecipeDef / ShopDef / LootTableDef
         EncounterDef / SpawnGroupDef / ProgressionDef / DifficultyDef
         ↑ 依赖 L0-L2

L2:      Entity（游戏实体层）
         CharacterDef / MonsterDef / ItemDef / EquipmentDef
         ConsumableDef / SummonDef
         ↑ 依赖 L0-L1

L1:      Capability（能力规则层）
         ConditionDef / TriggerDef / TargetingDef / EffectDef
         BuffDef / ModifierDef / ExecutionDef / StackingDef / CueDef / AbilityDef
         ↑ 依赖 L0

         Def 定义详情见 `definitions/README.md`

L0:      Vocabulary（基础词汇层）
         TagDef / AttributeDef / DamageTypeDef / FactionDef
         ElementDef / StatusCategoryDef
         ↑ 无下层依赖
```

每层的详细定义见 `content-layering.md`。

### 2.1 层间依赖方向

```
L4 ──→ L3 ──→ L2 ──→ L1 ──→ L0
← 禁止反向 ────────────────────
```

- 高层 Def 可引用低层 Def（L4 引用 L3、L2、L1、L0）
- 低层 Def 禁止引用高层 Def（L2 不可引用 L3）
- 同层 Def 可互相引用（L1 EffectDef 引用 L1 ConditionDef）
- 违反层间依赖规则 = 编译模式校验失败

### 2.2 各层与 Capabilities/Domains 的对应

| 内容层 | 对应 Capabilities | 对应 Domains |
|--------|-------------------|-------------|
| L0 Vocabulary | Tag, Attribute, Element | Faction, Terrain (基础类型) |
| L1 Capability | Condition, Trigger, Targeting, Effect, Modifier, Execution, Stacking, Cue, Ability, Spec | Spell |
| L2 Entity | — | Inventory, Party, Summon |
| L3 Gameplay | — | Quest, Economy, Crafting, Progression, Combat (Encounter) |
| L4 World | — | Narrative, Tactical (Map), Terrain (Region) |

---

## 3. 内容平台核心能力

### 3.1 Content Pipeline（加载管线）

```
assets/config/00_vocabulary/*.ron
assets/config/01_capabilities/*.ron         Load ──→ Deserialize ──→ Validate ──→ Register ──→ Freeze
assets/config/02_entities/*.ron                  ↑            ↑             ↑            ↑
assets/config/03_gameplay/*.ron              AssetServer   ron::from_str  Validator    DefRegistry
assets/config/04_world/*.ron
```

### 3.2 Registry 系统

- 通用泛型 `DefRegistry<T>` 作为所有 Def 类型的统一注册入口
- 支持按 ID 查询、按 Tag 过滤、遍历、计数
- 冻结后只读，不允许运行时修改
- 支持依赖查询：给定 Def ID，返回其引用的所有其他 Def ID

### 3.3 Validation 系统

每个 Content 加载通过 8 项强制校验：

| # | 校验项 | 说明 |
|---|--------|------|
| 1 | ID 唯一性 | 全局无重复 Def ID |
| 2 | 引用存在性 | 跨 Def 引用均指向已注册 Def |
| 3 | 循环检测 | 依赖图无循环 |
| 4 | 枚举有效性 | 枚举字段匹配已定义变体 |
| 5 | Tag 有效性 | 使用 Tag 均已注册 |
| 6 | 资产存在性 | 引用的 Icon/VFX/SFX 等资源存在 |
| 7 | Schema 兼容性 | fields 匹配当前 schema_version |
| 8 | Localization 完整性 | 所有 LocalizationKey 有对应条目 |

### 3.4 Content Dependency Graph

- 全量 Def 间的引用关系图
- 支撑：引用完整性检查、循环检测、影响分析（"修改 Fireball 会影响哪些 Def？"）
- 支撑：加载顺序推导（基于拓扑排序）
- 支撑：死内容检测（未被任何 Def 引用的孤立 Def）

---

## 4. 资产目录结构

### 4.1 assets/config/ 物理目录

```
assets/config/
├── 00_vocabulary/              # L0: 基础词汇
│   ├── tags.ron                # TagDefinition
│   ├── attributes.ron          # AttributeDefinition
│   ├── damage_types.ron        # DamageTypeDefinition
│   ├── factions.ron            # FactionDefinition
│   ├── elements.ron            # ElementDefinition
│   └── status_categories.ron   # StatusCategoryDefinition
│
├── 01_capabilities/            # L1: 能力规则
│   ├── conditions.ron          # ConditionDef
│   ├── triggers.ron            # TriggerDef
│   ├── targeting.ron           # TargetingDef
│   ├── effects.ron             # EffectDef
│   ├── modifiers.ron           # ModifierDef
│   ├── executions.ron          # ExecutionDef
│   ├── stackings.ron           # StackingDef
│   ├── cues.ron                # CueDef
│   └── abilities.ron           # AbilityDef / SpellDef
│
├── 02_entities/                # L2: 游戏实体
│   ├── characters.ron          # CharacterDef
│   ├── monsters.ron            # MonsterDef
│   ├── items.ron               # ItemDef
│   ├── equipment.ron           # EquipmentDef
│   ├── consumables.ron         # ConsumableDef
│   └── summons.ron             # SummonDef
│
├── 03_gameplay/                # L3: 玩法系统
│   ├── quests.ron              # QuestDef
│   ├── recipes.ron             # RecipeDef
│   ├── shops.ron               # ShopDef
│   ├── loot_tables.ron         # LootTableDef
│   ├── encounters.ron          # EncounterDef
│   ├── spawn_groups.ron        # SpawnGroupDef
│   ├── progression.ron         # ProgressionDef
│   └── difficulties.ron        # DifficultyDef
│
└── 04_world/                   # L4: 叙事世界
    ├── maps.ron                # MapDef
    ├── regions.ron             # RegionDef
    ├── scenes.ron              # SceneDef
    ├── cutscenes.ron           # CutsceneDef
    ├── narrative_arcs.ron      # NarrativeArcDef
    ├── story_flags.ron         # StoryFlagDef
    └── companions.ron          # CompanionDef
```

### 4.2 目录编号规则

目录前缀 `NN_` 表示层号：
- `00_vocabulary/` — L0
- `01_capabilities/` — L1
- `02_entities/` — L2
- `03_gameplay/` — L3
- `04_world/` — L4

编号保证按层序加载：L0 全部就绪后才能加载 L1，依次类推。

---

## 5. Def 类型按层完整清单

### L0: Vocabulary Defs

| Def 类型 | Rust 类型 | ID 类型 | 对应 domain |
|---------|-----------|---------|-----------|
| TagDefinition | `TagDef` | `TagId` | tag |
| AttributeDefinition | `AttributeDef` | `AttributeId` | attribute |
| DamageTypeDefinition | `DamageTypeDef` | `DamageTypeId` | combat |
| FactionDefinition | `FactionDef` | `FactionId` | faction |
| ElementDefinition | `ElementDef` | `ElementId` | combat/magic |
| StatusCategoryDefinition | `StatusCategoryDef` | `StatusCategoryId` | effect/stacking |

### L1: Capability Defs

| Def 类型 | Rust 类型 | ID 类型 | 对应 domain | Def 定义文档 |
|---------|-----------|---------|-----------|-------------|
| ConditionDef | `ConditionDef` | `ConditionId` | condition | `definitions/condition-def.md` |
| TriggerDef | `TriggerDef` | `TriggerId` | trigger | `definitions/trigger-def.md` |
| TargetingDef | `TargetingDef` | `TargetingId` | targeting | `definitions/targeting-def.md` |
| EffectDef | `EffectDef` | `EffectId` | effect | `definitions/effect-def.md` |
| BuffDef | `BuffDef` | `BuffId` | effect/stacking | `definitions/buff-def.md` |
| ModifierDef | `ModifierDef` | `ModifierId` | modifier | `definitions/modifier-def.md` |
| ExecutionDef | `ExecutionDef` | `ExecutionId` | execution | `definitions/execution-def.md` |
| StackingDef | `StackingDef` | `StackingId` | stacking | `definitions/stacking-def.md` |
| CueDef | `CueDef` | `CueId` | cue | `definitions/cue-def.md` |
| AbilityDef | `AbilityDef` | `AbilityId` | ability | `definitions/ability-def.md` |
| SpellDef | `SpellDef` | `SpellId` | spell | `definitions/ability-def.md` |

### L2: Entity Defs

| Def 类型 | Rust 类型 | ID 类型 | 对应 domain | Def 定义文档 |
|---------|-----------|---------|-----------|-------------|
| CharacterDef | `CharacterDef` | `CharacterId` | party | `definitions/entities/character-def.md` |
| MonsterDef | `MonsterDef` | `MonsterId` | combat/encounter | `definitions/entities/monster-def.md` |
| ItemDef | `ItemDef` | `ItemId` | inventory | `definitions/entities/item-def.md` |
| EquipmentDef | `EquipmentDef` | `EquipmentId` | inventory | `definitions/entities/equipment-def.md` |
| ConsumableDef | `ConsumableDef` | `ConsumableId` | inventory | `definitions/entities/consumable-def.md` |
| SummonDef | `SummonDef` | `SummonId` | summon | `definitions/entities/summon-def.md` |

### L3: Gameplay Defs

| Def 类型 | Rust 类型 | ID 类型 | 对应 domain |
|---------|-----------|---------|-----------|
| QuestDef | `QuestDef` | `QuestId` | quest |
| RecipeDef | `RecipeDef` | `RecipeId` | crafting |
| ShopDef | `ShopDef` | `ShopId` | economy |
| LootTableDef | `LootTableDef` | `LootTableId` | economy/combat |
| EncounterDef | `EncounterDef` | `EncounterId` | combat |
| SpawnGroupDef | `SpawnGroupDef` | `SpawnGroupId` | combat |
| ProgressionDef | `ProgressionDef` | `ProgressionId` | progression |
| DifficultyDef | `DifficultyDef` | `DifficultyId` | combat |

### L4: World Defs

| Def 类型 | Rust 类型 | ID 类型 | 对应 domain |
|---------|-----------|---------|-----------|
| MapDef | `MapDef` | `MapId` | tactical |
| RegionDef | `RegionDef` | `RegionId` | terrain |
| SceneDef | `SceneDef` | `SceneId` | narrative |
| CutsceneDef | `CutsceneDef` | `CutsceneId` | narrative |
| NarrativeArcDef | `NarrativeArcDef` | `NarrativeArcId` | narrative |
| StoryFlagDef | `StoryFlagDef` | `StoryFlagId` | narrative |
| CompanionDef | `CompanionDef` | `CompanionId` | party |

---

## 6. 与领域规则和数据 Schema 的映射

| 5层分类 | 领域规则 (02-domain) | 数据 Schema (04-data) | Content Def (03-content/definitions) |
|---------|---------------------|---------------------|-------------------------------------|
| **L0 Vocabulary** | `tag_domain.md`, `attribute_domain.md`, `faction_domain.md`, `combat_domain.md` (元素) | `tag_schema.md`, `attribute_schema.md`, `element_schema.md`, `status_category_schema.md`, 等 | `definitions/vocabulary/README.md` + 各 Def 文件 |
| **L1 Capability** | `condition_domain.md`, `trigger_domain.md`, `targeting_domain.md`, `effect_domain.md`, `modifier_domain.md`, `execution_domain.md`, `stacking_domain.md`, `cue_domain.md`, `ability_domain.md` | 各 capability 的 schema | `definitions/README.md` + 各 Def 文件 |
| **L2 Entity** | `inventory_domain.md`, `party_domain.md`, `summon_domain.md` | `inventory_schema.md`, `party_schema.md`, `summon_schema.md` | `definitions/entities/README.md` + 各实体 Def 文件 |
| **L3 Gameplay** | `quest_domain.md`, `economy_domain.md`, `crafting_domain.md`, `progression_domain.md`, `combat_domain.md` | `quest_schema.md`, `economy_schema.md`, `crafting_schema.md`, `progression_schema.md` | `definitions/gameplay/README.md` + 各玩法 Def 文件 |
| **L4 World** | `narrative_domain.md`, `tactical_domain.md`, `terrain_domain.md` | `narrative_schema.md`, `tactical_schema.md`, `terrain_schema.md` | TBD |

---

## 7. 文档结构

```
03-content/
├── README.md                          ← 本文（内容平台总纲）
├── content-layering.md                ← 5层分层体系详细定义
├── content-platform-manifesto.md      ← 内容平台核心理念与治理原则
└── definitions/                       ← L0-L4 Def 类型定义（Asset 结构、Registry、校验）
    ├── README.md                      ← L0-L3 Def 索引（Vocabulary + Capability + Entity + Gameplay）
    ├── vocabulary/                    ← L0 Vocabulary Def 子目录
    │   ├── README.md                  ← L0 层索引
    │   ├── tag-def.md                 ← TagDef 定义
    │   ├── attribute-def.md           ← AttributeDef 定义
    │   ├── damage-type-def.md         ← DamageTypeDef 定义
    │   ├── faction-def.md             ← FactionDef 定义
    │   ├── element-def.md             ← ElementDef 定义
    │   └── status-category-def.md     ← StatusCategoryDef 定义
    ├── effect-def.md                  ← EffectDef 定义
    ├── buff-def.md                    ← BuffDef 定义
    ├── modifier-def.md                ← ModifierDef 定义
    ├── condition-def.md               ← ConditionDef 定义
    ├── trigger-def.md                 ← TriggerDef 定义
    ├── targeting-def.md               ← TargetingDef 定义
    ├── execution-def.md               ← ExecutionDef 定义
    ├── stacking-def.md                ← StackingDef 定义
    ├── cue-def.md                     ← CueDef 定义
    ├── ability-def.md                 ← AbilityDef 定义（TODO）
    ├── entities/                      ← L2 Entity Def 子目录
    │   ├── README.md                  ← L2 Entity 索引
    │   ├── character-def.md           ← CharacterDef 定义
    │   ├── monster-def.md             ← MonsterDef 定义
    │   ├── item-def.md                ← ItemDef 定义
    │   ├── equipment-def.md           ← EquipmentDef 定义
    │   ├── consumable-def.md          ← ConsumableDef 定义
    │   └── summon-def.md              ← SummonDef 定义
    └── gameplay/                      ← L3 Gameplay Def 子目录
        ├── README.md                  ← L3 Gameplay 索引
        ├── quest-def.md               ← QuestDef 定义
        ├── recipe-def.md              ← RecipeDef 定义
        ├── shop-def.md                ← ShopDef 定义
        ├── loot-table-def.md          ← LootTableDef 定义
        ├── encounter-def.md           ← EncounterDef 定义
        ├── spawn-group-def.md         ← SpawnGroupDef 定义
        ├── progression-def.md         ← ProgressionDef 定义
        └── difficulty-def.md          ← DifficultyDef 定义
```

> 各 Def 类型的具体 Schema 定义归属 `docs/04-data/`（数据 Schema）和 `docs/02-domain/`（领域规则）。Def 定义文档（`definitions/`）是介于 Schema 和资产配置之间的桥接层——定义 Asset 结构、Registry 模式和校验规则。

---

## 8. 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `README.md` | ✅ updated | content-architect | 2026-06-20 |
| `content-layering.md` | ✅ updated | content-architect | 2026-06-20 |
| `content-platform-manifesto.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/README.md` | ✅ updated | content-architect | 2026-06-20 |
| `definitions/vocabulary/README.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/vocabulary/tag-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/vocabulary/attribute-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/vocabulary/damage-type-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/vocabulary/faction-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/vocabulary/element-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/vocabulary/status-category-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/effect-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/buff-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/modifier-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/condition-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/trigger-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/targeting-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/execution-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/stacking-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/cue-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/ability-def.md` | 🟡 TODO | content-architect | — |
| `definitions/entities/README.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/entities/character-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/entities/monster-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/entities/item-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/entities/equipment-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/entities/consumable-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/entities/summon-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/README.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/quest-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/recipe-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/shop-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/loot-table-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/encounter-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/spawn-group-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/progression-def.md` | ✅ new | content-architect | 2026-06-20 |
| `definitions/gameplay/difficulty-def.md` | ✅ new | content-architect | 2026-06-20 |

---

## 9. 关键设计决策索引

| 决策 | 文档 | 状态 |
|------|------|------|
| 5层内容分层体系 | `content-layering.md` | draft |
| 层间依赖方向（禁止反向） | `content-layering.md` Sec 7 | draft |
| Content Pipeline 管线 | `content-platform-manifesto.md` | draft |
| DefRegistry 泛型设计 | `content-platform-manifesto.md` | draft |
| 8项强制校验 | `content-platform-manifesto.md` | draft |
| Content Dependency Graph | `content-platform-manifesto.md` | draft |
| Content Ownership | `content-platform-manifesto.md` | draft |
| 资产目录编号约定 | `content-layering.md` Sec 8 | draft |

---

*本文档由 @content-architect 维护。所有 Content Platform 架构变更需经过 Content Architect 审查。底层架构决策应以 ADR 形式记录于 `docs/01-architecture/` 对应的 cross-cutting 目录。*
