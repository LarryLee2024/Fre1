---
id: 03-content.definitions.gameplay.quest-def
title: QuestDef — Quest Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# QuestDef — Quest Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/quest_domain.md` | **数据 Schema**: `docs/04-data/domains/quest_schema.md` | **插件代码**: `src/content/plugins/quest_plugin.rs`

---

## 1. Overview

QuestDef 定义了游戏中的任务模板——包括目标（Objective）、前置条件（Prerequisite）、奖励（Reward）、失败条件、任务链。QuestDef 是"任务配置"而非"任务实例"：每个玩家的任务状态（进行中/已完成/失败）由 Quest 领域的运行时状态管理。

### 关键设计原则

- **配置驱动**：QuestDef 定义任务的结构性数据（目标类型、数量、奖励），不包含运行时进度（如"已击杀 3/5 哥布林"）
- **L2 引用为主**：任务目标主要引用 L2 实体（Kill → MonsterDef, Collect → ItemDef, TalkTo → CharacterDef），不直接引用地图或场景
- **L4 软引用**：需要地图位置的目标（如 ReachLocation）使用字符串 `location_key`，不做强类型 Def 引用，遵守 L3 不可引用 L4 的层间规则
- **任务链**：`next_quests` 字段通过 QuestId 同层引用实现任务链（前置 Quest → 后置 Quest）
- **条件复用**：使用 L1 ConditionDef 表达失败条件、分支条件，不将条件逻辑硬编码在 QuestDef 中

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `quest_domain.md` | 任务状态机、目标类型、奖励结构 |
| `quest_schema.md` | QuestDef 完整字段结构、ObjectiveType 枚举 |
| `faction-def.md` | 本 Def 的 `prerequisites.faction_standing` 引用的 FactionDef |
| `condition-def.md` | 本 Def 的 `prerequisites`, `failure_conditions` 引用的 ConditionDef |
| `ability-def.md` | 本 Def 的 `rewards.unlock_abilities` 引用的 AbilityDef |
| `character-def.md` | 本 Def 的 `objectives[].TalkTo` 引用的 CharacterDef |
| `monster-def.md` | 本 Def 的 `objectives[].Kill` 引用的 MonsterDef |
| `item-def.md` | 本 Def 的 `rewards.items` 引用的 ItemDef |
| `equipment-def.md` | 本 Def 的 `rewards.items` 引用的 EquipmentDef |
| `consumable-def.md` | 本 Def 的 `rewards.items` 引用的 ConsumableDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 任务模板定义——描述一个可完成的任务/成就。
///
/// QuestDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<QuestDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct QuestDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（QuestDef 前缀: `qst_`）
    pub id: QuestId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── 任务类型 ──
    /// 任务分类（主线/支线/阵营/同伴/日常/成就）
    pub quest_type: QuestType,

    // ── 前置条件 ──
    /// 接受任务前必须满足的条件
    pub prerequisites: Vec<QuestPrerequisite>,

    // ── 任务目标 ──
    /// 需要完成的任务目标列表（全部完成 = 任务可交付）
    pub objectives: Vec<QuestObjective>,

    // ── 奖励 ──
    /// 任务完成时发放的奖励
    pub rewards: QuestRewards,

    // ── 失败条件 ──
    /// 任务失败条件（引用 L1 ConditionDef）
    /// 当任一条件满足时任务标记为 Failed
    pub failure_conditions: Option<Vec<ConditionId>>,

    // ── 任务链 ──
    /// 本任务完成后解锁的后继任务（同层引用 QuestDef）
    pub next_quests: Option<Vec<QuestId>>,

    // ── 接取与交付 ──
    /// 任务发放者 NPC（引用 L2 CharacterDef，可选）
    pub quest_giver: Option<CharacterId>,
    /// 任务交付者 NPC（引用 L2 CharacterDef，可选，与 quest_giver 不同时）
    pub quest_turn_in: Option<CharacterId>,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef，用于分类过滤）
    pub tags: Vec<TagId>,
    /// 任务图标 Key
    pub icon_key: Option<String>,

    // ── L4 软引用 ──
    /// 关联的地图位置 key（L4 软引用，非强类型 Def 引用）
    ///
    /// 此字段用于在 L4 侧建立 Quest → Map 的关联。
    /// L4 MapDef 通过此 key 标记该任务相关的触发位置。
    /// 具体设计在 L4 World 层完成时补充。
    pub location_hint: Option<String>,
}
```

### 内嵌数据结构

```rust
/// 任务前置条件——接受任务前必须满足的条件组合
#[derive(Deserialize, Clone, Debug)]
pub struct QuestPrerequisite {
    /// 前置条件的具体类型
    pub prerequisite_type: PrerequisiteType,
    /// 条件描述（可选，当需要向玩家展示"为什么不可接受"时使用）
    pub description_key: Option<LocalizationKey>,
}

/// 前置条件类型枚举——支持 AND (All)/OR (Any) 组合
#[derive(Deserialize, Clone, Debug)]
pub enum PrerequisiteType {
    /// 需要完成前置任务（引用 QuestDef）
    PreviousQuest(QuestId),
    /// 需要最低角色等级
    MinimumLevel(u32),
    /// 需要特定阵营声望值
    FactionStanding(FactionId, i32),
    /// 需要拥有特定物品（引用 ItemDef/EquipmentDef/ConsumableDef）
    HasItem(PolymorphicItemRef),
    /// 需要满足特定 L1 ConditionDef
    Condition(ConditionId),
    /// 需要满足所有子条件（AND）
    All(Vec<QuestPrerequisite>),
    /// 需要满足任一子条件（OR）
    Any(Vec<QuestPrerequisite>),
}

/// 多态物品引用——代表 ItemDef/EquipmentDef/ConsumableDef 中的一种
#[derive(Deserialize, Clone, Debug)]
pub enum PolymorphicItemRef {
    Item(ItemId),
    Equipment(EquipmentId),
    Consumable(ConsumableId),
}

/// 任务目标——构成任务进度追踪的基本单元
#[derive(Deserialize, Clone, Debug)]
pub struct QuestObjective {
    /// 目标类型（决定进度追踪的逻辑）
    pub objective_type: ObjectiveType,
    /// 目标数量（达到此值时目标完成）
    pub quantity: u32,
    /// 进度追踪 Key（可选，用于区分同类型目标的进度存储）
    pub progress_key: Option<String>,
    /// 目标描述（本地化 Key）
    pub description_key: LocalizationKey,
    /// 是否可选（true = 可选目标，不影响任务完成但影响额外奖励）
    pub optional: bool,
    /// 是否隐藏（true = 对玩家不可见，用于隐藏式目标）
    pub hidden: bool,
}

/// 任务目标类型枚举
#[derive(Deserialize, Clone, Debug)]
pub enum ObjectiveType {
    /// 击杀指定怪物类型（引用 MonsterDef）
    Kill(MonsterId),
    /// 收集指定物品（引用 ItemDef/EquipmentDef/ConsumableDef）
    Collect(PolymorphicItemRef),
    /// 与指定 NPC 对话（引用 CharacterDef）
    TalkTo(CharacterId),
    /// 到达指定位置（L4 软引用，location_key 字符串）
    ReachLocation {
        /// 位置 key（L4 软引用，由 L4 MapDef 关联定义）
        location_key: String,
    },
    /// 在指定目标上使用物品
    UseItemOnTarget {
        /// 要使用的物品（引用 ItemDef/ConsumableDef）
        item_ref: PolymorphicItemRef,
        /// 使用目标（角色/怪物/位置）
        target: ObjectiveTarget,
    },
    /// 护送 NPC 到达目的地
    Escort {
        /// 被护送 NPC（引用 CharacterDef）
        npc_id: CharacterId,
        /// 目的地位置 key（L4 软引用）
        destination_key: String,
    },
    /// 自定义条件（通过 L1 ConditionDef 表达任意进度逻辑）
    Custom(ConditionId),
    /// 需要完成所有子目标（AND）
    All(Vec<QuestObjective>),
    /// 需要完成任一子目标（OR）
    Any(Vec<QuestObjective>),
}

/// 目标操作对象——描述 UseItemOnTarget 的目标
#[derive(Deserialize, Clone, Debug)]
pub enum ObjectiveTarget {
    /// 对指定角色使用（引用 CharacterDef）
    Character(CharacterId),
    /// 对指定怪物类型使用（引用 MonsterDef）
    Monster(MonsterId),
    /// 在指定位置使用（L4 软引用）
    Location(String),
}

/// 任务奖励——任务完成时发放的全部奖励
#[derive(Deserialize, Clone, Debug)]
pub struct QuestRewards {
    /// 经验值奖励
    pub xp: u32,
    /// 金币奖励
    pub gold: u32,
    /// 物品奖励列表（多态引用 L2 物品类 Def）
    pub items: Option<Vec<QuestRewardItem>>,
    /// 阵营声望奖励（(faction_id, 声望值变化)）
    pub faction_reputation: Option<Vec<(FactionId, i32)>>,
    /// 奖励获得的能力（引用 L1 AbilityDef）
    pub unlock_abilities: Option<Vec<AbilityId>>,
    /// 奖励解锁的配方（引用 L3 RecipeDef）
    pub unlock_recipes: Option<Vec<RecipeId>>,
    /// 奖励解锁的商店（引用 L3 ShopDef）
    pub unlock_shops: Option<Vec<ShopId>>,
}

/// 任务奖励物品（带数量和多态引用）
#[derive(Deserialize, Clone, Debug)]
pub struct QuestRewardItem {
    pub item: PolymorphicItemRef,
    pub quantity: u32,
}

/// 任务类型枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum QuestType {
    /// 主线任务
    Main,
    /// 支线任务
    Side,
    /// 阵营任务（可重复）
    Faction,
    /// 同伴个人任务
    Companion,
    /// 日常任务
    Daily,
    /// 成就（一次性，完成即标记）
    Achievement,
}
```

### 字段说明

- **`objectives`**: 任务是"全部目标完成 = 可交付"模型。目标通过 `All`/`Any` 支持嵌套组合（如"收集 5 个龙鳞 AND（杀死巨龙 OR 偷走龙蛋）"）
- **`failure_conditions`**: 当任一 ConditionId 满足时任务标记失败。不设置则任务不会自动失败
- **`prerequisites`**: 前置条件通过嵌套 `All`/`Any` 支持复杂逻辑（如"完成前置任务 AND（达到 10 级 OR 阵营声望尊敬）"）
- **`next_quests`**: 支持线性任务链（单个后继）和分支任务链（多个后继供选择）
- **`location_hint`**: L4 软引用。此字段不直接引用 MapDef，而是作为 L4 MapDef 反向关联 QuestDef 的匹配 key

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// QuestDef 注册插件
pub struct QuestDefPlugin;

impl Plugin for QuestDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<QuestDef>();
        app.init_asset_loader::<RonAssetLoader<QuestDef>>();
        app.insert_resource(DefRegistry::<QuestDef>::new());
        app.add_systems(
            PreUpdate,
            load_quest_defs
                .run_if(resource_changed::<Assets<QuestDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 QuestDef
pub fn get_quest_def(
    quest_id: &QuestId,
    registry: &DefRegistry<QuestDef>,
) -> Option<&QuestDef> {
    registry.get(quest_id)
}

/// 按任务类型过滤 QuestDef
pub fn get_quest_defs_by_type(
    quest_type: QuestType,
    registry: &DefRegistry<QuestDef>,
) -> Vec<&QuestDef> {
    registry.iter()
        .filter(|def| def.quest_type == quest_type)
        .collect()
}

/// 按标签过滤 QuestDef
pub fn get_quest_defs_by_tag(
    tag_id: &TagId,
    registry: &DefRegistry<QuestDef>,
) -> Vec<&QuestDef> {
    registry.iter()
        .filter(|def| def.tags.iter().any(|t| t == tag_id))
        .collect()
}
```

### 注册生命周期

```
QuestDefPlugin::build
  │
  ├── QuestDef 从 assets/config/03_gameplay/quests.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │     └── 校验: RON 语法正确性、枚举合法性
  │
  ├── Validate
  │     ├── ID 唯一性
  │     ├── L0-L2 引用存在性检查
  │     ├── L3 同层引用存在性（next_quests, unlock_recipes, unlock_shops）
  │     ├── 依赖图循环检查（next_quests 不得形成 A→B→A 循环）
  │     ├── 目标组合逻辑检查（All/Any 嵌套深度限制）
  │     ├── 奖励数值范围检查（xp, gold 合理范围）
  │     ├── L4 层禁止引用检查（无强类型 L4 引用）
  │     └── L4 软引用（location_key）记录但不校验
  │
  ├── Register（注入 DefRegistry<QuestDef>）
  │
  └── Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | QuestId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `objectives` 非空 | 任务必须有至少一个目标 |
| V4 | `quest_type` 合法 | 必须匹配 QuestType 的已知变体 |
| V5 | `xp` 和 `gold` 范围 | 0-10,000,000，合理上限 |
| V6 | `prerequisites` 嵌套深度 | All/Any 嵌套不超过 5 层 |
| V7 | `objectives` 嵌套深度 | All/Any 嵌套不超过 5 层 |
| V8 | `quantity` >= 1 | 目标数量至少为 1 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | `prerequisites` 中的每个 ConditionId 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V10 | `prerequisites` 中的每个 QuestId 已注册 | 在 DefRegistry<QuestDef> 中存在 |
| V11 | `failure_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V12 | `objectives` 中的每个 MonsterId (Kill) 已注册 | 在 DefRegistry<MonsterDef> 中存在 |
| V13 | `objectives` 中的每个 CharacterId (TalkTo/Escort) 已注册 | 在 DefRegistry<CharacterDef> 中存在 |
| V14 | `objectives` 中的每个 PolymorphicItemRef 已注册 | 在对应 DefRegistry 中存在 |
| V15 | `next_quests` 中的每个 QuestId（如果设置）已注册 | 在 DefRegistry<QuestDef> 中存在 |
| V16 | `quest_giver` 和 `quest_turn_in`（如果设置）已注册 | 在 DefRegistry<CharacterDef> 中存在 |
| V17 | `rewards` 中的每个 FactionId 已注册 | 在 DefRegistry<FactionDef> 中存在 |
| V18 | `rewards` 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V19 | `rewards` 中的每个 RecipeId 已注册 | 在 DefRegistry<RecipeDef> 中存在 |
| V20 | `rewards` 中的每个 ShopId 已注册 | 在 DefRegistry<ShopDef> 中存在 |
| V21 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V22 | QuestDef 不得引用任何 L4 World Def | 层间依赖方向规则 |
| V23 | QuestDef 不得包含强类型的 L4 ID 字段 | `location_hint` 是 String，非 MapId/SceneId |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V24 | `next_quests` 不得形成循环 | 任务链必须是 DAG（有向无环图） |
| V25 | `quest_type = Achievement` 时 `prerequisites` 应非空 | 成就不应接取即有 |
| V26 | 奖励合理性 | `xp` 和 `gold` 与 quest_type 和等级区间匹配 |
| V27 | 目标类型与引用一致性 | Kill 目标必须引用 MonsterDef，TalkTo 必须引用 CharacterDef |

### 4.5 L4 Soft Reference 校验

| # | 规则 | 说明 |
|---|------|------|
| V28 | `location_hint` 在 L3 加载时仅记录不校验 | L4 加载后二次解析 |
| V29 | `objectives` 中的 `location_key` 和 `destination_key` 类似处理 | L4 加载后验证所有 key 有对应定义 |

---

## 5. RON 示例

```ron
(
    id: "qst:dragon_slayer",
    name_key: "quest.qst_dragon_slayer.name",
    description_key: "quest.qst_dragon_slayer.desc",
    schema_version: 1,

    quest_type: Main,

    prerequisites: [
        (prerequisite_type: PreviousQuest("qst:ancient_secret")),
        (prerequisite_type: MinimumLevel(10)),
    ],

    objectives: [
        (
            objective_type: Kill("mob:dragon_elder"),
            quantity: 1,
            progress_key: None,
            description_key: "quest.qst_dragon_slayer.obj_kill_dragon",
            optional: false,
            hidden: false,
        ),
        (
            objective_type: Collect(Item("itm:dragon_scale")),
            quantity: 5,
            progress_key: None,
            description_key: "quest.qst_dragon_slayer.obj_collect_scales",
            optional: false,
            hidden: false,
        ),
        (
            objective_type: ReachLocation(
                location_key: "loc:dragon_peak_throne",
            ),
            quantity: 1,
            progress_key: None,
            description_key: "quest.qst_dragon_slayer.obj_reach_throne",
            optional: true,
            hidden: false,
        ),
    ],

    rewards: (
        xp: 5000,
        gold: 2000,
        items: Some([
            (item: Equipment("equip:dragon_slayer_sword"), quantity: 1),
            (item: Item("itm:dragon_scale"), quantity: 3),
        ]),
        faction_reputation: Some([
            ("faction:kingdom", 500),
            ("faction:hunters_guild", 200),
        ]),
        unlock_abilities: Some(["ability:dragon_roar"]),
        unlock_recipes: Some(["recipe:dragon_scale_armor"]),
        unlock_shops: None,
    ),

    failure_conditions: Some([
        "cond:is_dragon_alive_after_escaped",
    ]),

    next_quests: Some(["qst:dragon_war"]),

    quest_giver: Some("chr:king_arthur"),
    quest_turn_in: Some("chr:king_arthur"),

    tags: ["tag:main_quest", "tag:dragon", "tag:combat"],

    icon_key: Some("icons/quests/dragon_slayer.png"),

    location_hint: Some("loc:dragon_peak_throne"),
)
```

---

## 6. 与 EncounterDef / ShopDef / RecipeDef 的关系

QuestDef 是 L3 层的"编排中心"——它引用其他 L3 Def 来构建完整的任务体验：

| L3 Def | QuestDef 如何引用 |
|--------|------------------|
| RecipeDef | 通过 `rewards.unlock_recipes`，任务完成后解锁配方 |
| ShopDef | 通过 `rewards.unlock_shops`，任务完成后解锁商店访问权限 |
| QuestDef 自身 | 通过 `prerequisites.PreviousQuest` 和 `next_quests` 构建任务链 |
| EncounterDef | 不直接引用。L4 SceneDef 或 MapDef 定义哪个 Encounter 触发哪个 Quest 目标 |

**特殊规则**：
- QuestDef 引用 RecipeDef/ShopDef 是单向知识——任务知道它解锁了哪些商店，但 RecipeDef 和 ShopDef 不知道哪些任务解锁它们。任务进度需要解锁商店时，Quest 领域发送事件通知 Shop 领域解锁
- 任务链（`next_quests`）必须是 DAG。环形链（A→B→C→A）在校验时被检测并拒绝

---

*本文档由 @content-architect 维护。*
