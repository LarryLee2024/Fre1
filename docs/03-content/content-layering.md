---
id: 03-content.content-layering
title: Content Layering — 五层内容分层体系
status: draft
owner: content-architect
created: 2026-06-20
tags:
  - content
  - layering
  - dependency
  - architecture
  - l0
  - l1
  - l2
  - l3
  - l4
---

# Content Layering — 五层内容分层体系

> **职责**: @content-architect | **适用**: 所有 `assets/config/` 下的 RON 资产
> **核心规则**: 高层可引用低层，低层不可引用高层（单向依赖，禁止反向）

---

## 1. 设计目的

大型 SRPG 的内容量可达 10,000+ Def 资产。没有分层的扁平结构会导致：

- **循环引用**：ItemDef 引用 QuestDef，QuestDef 又引用 ItemDef，加载管线无法确定顺序
- **隐含耦合**：修改一个基础 Tag 需要检查所有引用它的 Quest、Cutscene、Shop
- **Mod 冲突**：没有层次约束，Mod 可引用任意内容，卸载时产生悬空引用
- **责任模糊**：EffectDef 和 CharacterDef 混在同一目录，新开发者不知道属于哪一层

5层分层体系通过**严格层间依赖方向**解决了上述问题。分层不是技术分类（"把 Entity 放在一起"），而是**内容抽象层级的自然划分**——从原子词汇到具体世界，每层定义自己的抽象边界。

---

## 2. L0: Vocabulary（基础词汇层）

### 2.1 职责

定义游戏世界的**基础词汇**——最小的、不可再分的语义单元。这一层不包含任何"逻辑"，只定义可被更高层引用的命名空间。

### 2.2 哲学依据

正如自然语言有词汇表，游戏世界也需要一套**基础概念集合**。L0 定义了"这个世界由什么构成"——元素类型（Fire、Ice）、生物分类（Humanoid、Beast）、阵营（Player、Enemy）、表面类型（Lava、Ice）。没有 L0，高层 Def 将各自发明自己的分类体系，导致语义不一致。

### 2.3 包含的 Def 类型

| Def 类型 | 用途 | 示例 ID |
|---------|------|---------|
| `TagDef` | 标签体系：元素、种族、职业、难度、物品类别 | `tag:fire`, `tag:humanoid`, `tag:healing` |
| `AttributeDef` | 属性定义：生命、攻击、防御、速度 | `attr:max_hp`, `attr:attack`, `attr:defense` |
| `DamageTypeDef` | 伤害类型：物理、火焰、冰冻、暗影 | `dmg:physical`, `dmg:fire`, `dmg:shadow` |
| `FactionDef` | 阵营定义：玩家、敌方、中立 | `faction:player`, `faction:enemy`, `faction:neutral` |
| `ElementDef` | 元素属性：火焰、冰冻、闪电、暗影 | `elem:fire`, `elem:ice`, `elem:lightning` |
| `StatusCategoryDef` | 状态类别：增益、减益、控制 | `status:physical_harmful`, `status:magical_harmful`, `status:control` |

### 2.4 设计约束

- L0 Def 不允许引用任何其他 Def（无下层依赖）
- L0 Def 只定义固定的 ID 和元数据（Icon、Name、Description）
- L0 初始化顺序决定所有上层内容的加载前提
- 不建议超过 200 个 L0 Def（词汇量不宜膨胀）

---

## 3. L1: Capability（能力规则层）

### 3.1 职责

定义游戏机制的**可复用规则模块**——不出现具体技能或角色，只出现可以被组合的规则片段。L1 是"游戏机制的乐高积木"。

### 3.2 哲学依据

L1 是整个 Content Platform 的设计核心。传统 SRPG 把 Fireball 写成一个硬编码技能（"Fireball 造成 3d6 火焰伤害"），然后在游戏里复制粘贴出 Iceball、LightningBolt、AcidArrow。这种模式的代价：

- 游戏效果不可复用——没法定义一个"每回合造成火焰伤害"的通用效果，然后在 Fireball、FireWall、BurningGround 之间共享
- 数据膨胀——N 个技能 × M 个效果 = N×M 个数据条目，而不是 N+M 个可组合模块
- 平衡调整困难——改火焰伤害系数需要改所有引用了火焰伤害的技能

L1 通过**组合而非继承**解决：EffectDef 定义"每回合造成X类型伤害"，AbilityDef 引用 EffectDef + TargetingDef + ConditionDef + CueDef。FireWall 和 Fireball 共享同一个 BurnEffect，只是 Target 范围和持续时间不同。

### 3.3 包含的 Def 类型

| Def 类型 | 用途 | 示例 ID |
|---------|------|---------|
| `ConditionDef` | 条件检查：生物类型判断、属性对比、Tag 检查 | `cond:is_enemy`, `cond:has_tag_fire` |
| `TriggerDef` | 触发模式：命中时、回合开始时、受击时 | `trig:on_hit`, `trig:on_turn_start`, `trig:on_damage_taken` |
| `TargetingDef` | 目标选择：范围形状、目标过滤 | `tgt:cone_3`, `tgt:single_enemy`, `tgt:radius_2_ally` |
| `EffectDef` | 效果逻辑：持续伤害、治疗、属性修改 | `eff:burn`, `eff:heal`, `eff:stun`, `eff:fire_damage` |
| `ModifierDef` | 数值修饰：百分比增幅、固定值修改 | `mod:fire_damage_pct`, `mod:defense_flat` |
| `ExecutionDef` | 执行计算：伤害公式、治疗公式 | `exec:melee_damage`, `exec:magic_heal` |
| `StackingDef` | 堆叠规则：不叠加、取最高、累加 | `stk:unstackable`, `stk:strongest`, `stk:additive` |
| `CueDef` | 表现信号：VFX、SFX、Camera 行为 | `cue:explosion_fire`, `cue:heal_glow` |
| `AbilityDef` | 技能模板：组合上述 Def 为可执行的技能 | `ability:fireball`, `ability:heal_wound` |

### 3.4 设计约束

- L1 Def 可引用 L0 和同层 Def（EffectDef 引用了 DamageTypeDef + ModifierDef）
- L1 禁止引用 L2+ Def（EffectDef 不可引用 CharacterDef）
- AbilityDef 是 L1 的组合终端——它将 Condition/Targeting/Effect/Cue 组装为一个完整的"能力"

---

## 4. L2: Entity（游戏实体层）

### 4.1 职责

定义**游戏中出现的具体实体**——角色、怪物、物品、召唤物。L2 是"可放入世界的具体事物"。

### 4.2 哲学依据

L1 定义了"燃烧效果"（BurnEffect），L2 定义了"被燃烧效果影响的具体实体"——比如 FireMage 角色有一个 Fireball 能力，Fireball 引用 BurnEffect。L2 是**能力的载体**。

L2 不定义"这个角色在哪个地图出现"或"这个任务需要什么物品"——那些属于 L3/L4。L2 只定义实体本身的能力组合和属性基线。

### 4.3 包含的 Def 类型

| Def 类型 | 用途 | 示例 ID |
|---------|------|---------|
| `CharacterDef` | 玩家角色：属性基线、可用能力、职业、擅长武器 | `char:fire_mage`, `char:shadow_rogue` |
| `MonsterDef` | 怪物模板：属性、技能组、掉落表引用、AI 行为 | `mon:dragon_ancient`, `mon:goblin_archer` |
| `ItemDef` | 普通物品（非装备/消耗品）：任务道具、材料 | `item:dragon_scale`, `item:herb` |
| `EquipmentDef` | 可装备物品：武器、防具、饰品 | `equip:flame_sword`, `equip:dragon_armor` |
| `ConsumableDef` | 消耗品：药水、卷轴、投掷物 | `cons:health_potion`, `cons:firebomb` |
| `SummonDef` | 召唤物模板：属性、持续时间、技能组 | `summon:fire_elemental`, `summon:skeleton_warrior` |

### 4.4 设计约束

- L2 Def 可引用 L0、L1 和同层 Def
- CharacterDef 可引用 TagDef、AbilityDef、EquipmentDef
- L2 禁止引用 L3+ Def（ItemDef 不可引用 QuestDef）

---

## 5. L3: Gameplay（玩法系统层）

### 5.1 职责

定义**驱动游戏进程的系统**——任务、商店、掉落、遭遇战、难度曲线。L3 是"让游戏运转起来的规则系统"。

### 5.2 哲学依据

L3 是传统内容设计中**最容易被遗漏**的层级。很多项目把 QuestDef 放在"任务"目录下，把 RecipeDef 放在"制造"目录下，但它们共享一个核心特征：它们不是"实体"而是**系统配置**。QuestDef 定义了"收集 5 个 DragonScale 并交给 NPC"，它依赖 L2 的 ItemDef（DragonScale）和 L0 的 FactionDef（NPC 阵营），但它自己属于更高的抽象层级——它描述的是**游戏状态的变化目标**，而非"什么东西存在"。

### 5.3 包含的 Def 类型

| Def 类型 | 用途 | 示例 ID |
|---------|------|---------|
| `QuestDef` | 任务/成就：目标、条件、阶段、奖励 | `quest:dragon_slayer`, `quest:herb_collection` |
| `RecipeDef` | 制造配方：输入物品、输出物品、技能要求 | `recipe:health_potion`, `recipe:flame_sword` |
| `ShopDef` | 商店配置：商品列表、价格系数、阵营限制 | `shop:blacksmith`, `shop:alchemist` |
| `LootTableDef` | 掉落表：概率权重、条件、稀有度 | `loot:dragon_hoard`, `loot:goblin_camp` |
| `EncounterDef` | 遭遇战配置：怪物组合、位置、触发条件 | `enc:forest_ambush`, `enc:boss_fight_01` |
| `SpawnGroupDef` | 生成组：怪物类型、数量、重生 | `spawn:dungeon_floor_1`, `spawn:wolf_pack` |
| `ProgressionDef` | 成长曲线：升级经验表、职业解锁条件 | `prog:level_curve`, `prog:class_unlock` |
| `DifficultyDef` | 难度配置：全局数值倍率、AI 强度 | `diff:normal`, `diff:hard`, `diff:nightmare` |

### 5.4 设计约束

- L3 Def 可引用 L0、L1、L2 和同层 Def
- QuestDef 可引用 ItemDef、CharacterDef、FactionDef、ShopDef
- L3 禁止引用 L4 Def（EncounterDef 不可引用 MapDef——地图关联通过 ID 引用从 L4 侧定义）

**特别说明：L3 引用 L4 的禁止规则**

地图上的遭遇战配置（哪个 Encounter 出现在哪个 Map）在 L4 MapDef 中定义，而非在 L3 EncounterDef 中定义。这样 L3 保持地图无关，L4 负责编排。违反此规则会导致：改地图配置需要同时改 EncounterDef，产生双向耦合。

---

## 6. L4: World（叙事世界层）

### 6.1 职责

定义**具体世界呈现**——地图、场景、对话、叙事弧。L4 是"玩家最终体验到的内容"。

### 6.2 哲学依据

L4 是内容量最大、变更最频繁的层。类比《博德之门3》，一个地图可能有数百个 SceneDef 和 StoryFlagDef。L4 的设计目标：

- **叙事内容与玩法规则解耦**：改对话脚本不需要改 EffectDef 或 CharacterDef
- **世界编排**：L4 负责"谁在哪里、何时出现、触发什么"，不负责"谁有什么能力"
- **Mod 友好**：Mod 的主要改动是 L4（新地图、新对话），极少需要改 L1-L2

### 6.3 包含的 Def 类型

| Def 类型 | 用途 | 示例 ID |
|---------|------|---------|
| `MapDef` | 单张地图/关卡：尺寸、Tile 层、物体、遭遇位置 | `map:dragon_peak`, `map:dark_forest` |
| `RegionDef` | 区域定义：地图集合、通行条件、世界地图位置 | `region:fire_kingdom`, `region:shadow_lands` |
| `SceneDef` | 场景定义：对话、叙事事件、脚本触发 | `scene:dragon_appears`, `scene:village_intro` |
| `CutsceneDef` | 过场动画：镜头、角色动画、对话时间线 | `cut:prologue`, `cut:final_battle` |
| `NarrativeArcDef` | 叙事弧：场景链、条件分支、结局 | `arc:main_quest`, `arc:companion_side` |
| `StoryFlagDef` | 故事标记：全局状态标记、前置条件 | `flag:killed_dragon`, `flag:joined_guild` |
| `CompanionDef` | 同伴配置：加入条件、离队条件、个人任务链 | `comp:shadow_rogue`, `comp:fire_mage` |

### 6.4 设计约束

- L4 Def 可引用所有下层 Def（L0-L3）和同层 Def
- MapDef 可引用 EncounterDef（L3）、ItemDef（L2）、FactionDef（L0）
- L4 无法被下层引用——没有 Def 低于 L4
- L4 是 Content Pipeline 最后加载的层

---

## 7. 层间依赖规则

### 7.1 核心规则

```
┌─────────────────────────────────────────────┐
│  高层可以引用低层                             │
│  L4 → L3, L4 → L2, L4 → L1, L4 → L0        │
│  L3 → L2, L3 → L1, L3 → L0                  │
│  L2 → L1, L2 → L0                            │
│  L1 → L0                                      │
│  L0 → (无)                                    │
├─────────────────────────────────────────────┤
│  低层禁止引用高层（红线）                      │
│  L0 -x→ L1, L0 -x→ L2, L0 -x→ L3, L0 -x→ L4 │
│  L1 -x→ L2, L1 -x→ L3, L1 -x→ L4            │
│  L2 -x→ L3, L2 -x→ L4                        │
│  L3 -x→ L4                                    │
└─────────────────────────────────────────────┘
```

### 7.2 层间引用示例（合法）

```
L4 MapDef
  ├──→ L3 EncounterDef（该地图上的遭遇战）
  ├──→ L2 CharacterDef（地图上的 NPC）
  ├──→ L1 AbilityDef（NPC 使用的能力）
  └──→ L0 TagDef（地图标签如 "dungeon", "forest"）

L3 QuestDef
  ├──→ L2 ItemDef（任务需要的物品）
  ├──→ L1 ConditionDef（完成条件）
  └──→ L0 FactionDef（任务涉及的阵营）

L2 CharacterDef
  ├──→ L1 AbilityDef（角色拥有的能力）
  ├──→ L1 EffectDef（常驻效果）
  └──→ L0 TagDef（角色标签）
```

### 7.3 层间引用示例（违规）

```
L0 TagDef -x→ L1 AbilityDef（基础词汇不可引用能力）
L1 EffectDef -x→ L2 CharacterDef（能力规则不可引用具体角色）
L2 ItemDef -x→ L3 QuestDef（实体不可引用任务）
L3 EncounterDef -x→ L4 MapDef（玩法系统不可引用具体地图）
```

### 7.4 同层引用

同层 Def 可以互相引用，但有约束：

- **L0**: 禁止同层引用（L0 Def 必须自包含）
- **L1**: 允许有限同层引用（EffectDef 引用 ConditionDef、ModifierDef）
- **L2**: 允许同层引用（CharacterDef 引用 EquipmentDef）
- **L3**: 允许同层引用（QuestDef 引用 ShopDef、RecipeDef）
- **L4**: 允许同层引用（MapDef 引用 SceneDef、NarrativeArcDef 引用 StoryFlagDef 和 SceneDef）

任何同层引用不得形成循环依赖。

---

## 8. 目录结构映射

### 8.1 物理目录映射

```
assets/config/
├── 00_vocabulary/              ← L0
├── 01_capabilities/            ← L1
├── 02_entities/                ← L2
├── 03_gameplay/                ← L3
└── 04_world/                   ← L4
```

### 8.2 目录编号规则

- `NN_` 前缀表示层号（00, 01, 02, 03, 04）
- 编号越小加载越早
- 编号不连续是为了未来可能的层间插入保留空间
- 同层内的文件按字母序加载（由加载管线的拓扑排序最终决定）

### 8.3 目录内的文件组织

每层目录内的文件组织遵循**单文件多 Def** 原则：

```
01_capabilities/
├── effects.ron        ← 所有 EffectDef 放在一个文件（而非每个 Def 一个文件）
├── conditions.ron
├── triggers.ron
├── targeting.ron
├── modifiers.ron
├── executions.ron
├── stackings.ron
├── cues.ron
└── abilities.ron
```

**理由**：
- 减少文件系统 I/O（Bevy AssetServer 加载 100 个文件 vs 1 个文件差异显著）
- 便于内容作者横向对比（所有 EffectDef 在一个文件中可以对比参数）
- 便于 Git 审查（改一个 Effect 不再需要开 50 个文件的 diff）

**例外**：当单个文件超过 2000 行或包含超过 50 个 Def 时，可按子目录拆分：

```
01_capabilities/effects/
├── damage.ron           ← 伤害类效果
├── healing.ron          ← 治疗类效果
├── control.ron          ← 控制类效果（眩晕、魅惑等）
└── utility.ron          ← 辅助类效果（护盾、隐身等）
```

---

## 9. 加载顺序

### 9.1 按层加载

Content Pipeline 严格按照层序加载：

```
Phase 1: Load L0 (Vocabulary)      → 注册所有 TagDef, AttributeDef, DamageTypeDef ...
Phase 2: Load L1 (Capability)      → 注册所有 ConditionDef, EffectDef, AbilityDef ...
Phase 3: Load L2 (Entity)          → 注册所有 CharacterDef, ItemDef ...
Phase 4: Load L3 (Gameplay)        → 注册所有 QuestDef, RecipeDef, ShopDef ...
Phase 5: Load L4 (World)           → 注册所有 MapDef, SceneDef, CutsceneDef ...
```

### 9.2 层内加载

同层内的加载顺序由 Content Dependency Graph 的拓扑排序决定：
- 一个 EffectDef 如果引用了另一个 EffectDef，被引用的先加载
- 层内循环引用 → Validation 报错并阻止层完成加载
- 同层无依赖的 Def 可并行加载

### 9.3 层间依赖检查

每层加载完成后执行层间引用校验：
- L1 加载完成后：验证所有 L1 Def 的跨层引用仅指向 L0（不指向 L2+）
- L2 加载完成后：验证所有 L2 Def 的跨层引用仅指向 L0-L1（不指向 L3+）
- 以此类推

---

## 10. 与 Data Schema 的关系

| 内容层 | 对应的 Data Schema (04-data) |
|--------|------------------------------|
| L0 | `tag_schema.md`, `attribute_schema.md`, `faction_schema.md` 等 |
| L1 | `condition_schema.md`, `targeting_schema.md`, `effect_schema.md`, `ability_schema.md` 等 |
| L2 | `inventory_schema.md`, `party_schema.md`, `summon_schema.md` |
| L3 | `quest_schema.md`, `economy_schema.md`, `crafting_schema.md`, `progression_schema.md` |
| L4 | `narrative_schema.md`, `tactical_schema.md`, `terrain_schema.md` |

每个 Data Schema 文档需标注其归属的内容层（`content-layer: L1`），便于跨文档追溯。

---

## 附录: 层迁移指南

当现有 Def 在错误的层定义时：

| 场景 | 处理方式 |
|------|---------|
| Effect 定义在 L2 实体层 | 迁移到 L1 capabilities，L2 引用迁移后的 L1 ID |
| Recipe 定义在 L1 能力层 | 迁移到 L3 gameplay，更新所有跨层引用 |
| Map 引用直接写在了 L3 Encounter 中 | 改为 L4 MapDef 引用 L3 EncounterDef |
| L0 Def 引用了 L1 Def | 破坏分层规则，必须通过参数化而非硬引用解决 |

层迁移必须同步更新：
1. 物理 RON 文件目录位置
2. 所有引用该 Def 的其他 Def
3. Content Dependency Graph 的边
4. 加载管线中的 Phase 分配

---

*本文档定义 Content Platform 的 5 层分层体系。所有新 Def 类型必须在这 5 层中找到归属层，未归属层的 Def 不可进入资产目录。分层规则的例外必须经过 @content-architect 批准并记录在案。*
