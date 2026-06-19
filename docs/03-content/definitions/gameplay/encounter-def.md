---
id: 03-content.definitions.gameplay.encounter-def
title: EncounterDef — Encounter Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# EncounterDef — Encounter Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/combat_domain.md` | **数据 Schema**: `docs/04-data/domains/combat_schema.md` | **插件代码**: `src/content/plugins/encounter_plugin.rs`

---

## 1. Overview

EncounterDef 定义了战斗遭遇战的配置——哪些怪物参与、如何触发、胜利/失败条件、难度覆盖和战利品加成。EncounterDef 是"遭遇战模板"而非"战斗实例"：每场战斗的运行时状态由 Combat 领域管理。

### 关键设计原则

- **生成组引用**：EncounterDef 引用 SpawnGroupDef 构建怪物编队，而非直接引用 MonsterDef。SpawnGroupDef 是可复用的组建模版，EncounterDef 负责编排和数量配置
- **L4 无关性**：EncounterDef 不包含地图位置信息。哪个地图发生哪个遭遇战由 L4 MapDef 定义，遵守 L3 不可引用 L4 的层间规则
- **条件触发**：`trigger_conditions` 使用 L1 ConditionDef 表达触发逻辑（如"当玩家进入特定区域"、"当 BOSS 被攻击后第二回合"）
- **胜负分离**：胜利条件和失败条件独立定义（vs 传统"消灭所有敌人 = 胜利，全灭 = 失败"的耦合），支持多样化的胜负场景
- **难度覆盖**：`difficulty_override` 引用 L3 DifficultyDef，允许单场遭遇战覆盖全局难度设定（如 Boss 战强制使用 Hard 难度）

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `combat_domain.md` | 战斗状态机、回合体系、胜负条件 |
| `combat_schema.md` | EncounterDef 完整字段结构、VictoryCondition/DefeatCondition 定义 |
| `condition-def.md` | 本 Def 的 `trigger_conditions` 引用的 ConditionDef |
| `ability-def.md` | 本 Def 的 `ai_config` 可能引用的 AbilityDef 标签 |
| `tag-def.md` | 本 Def 的 `environment_tags` 和 `tags` 引用的 TagDef |
| `monster-def.md` | 本 Def 间接通过 SpawnGroupDef 引用的 MonsterDef |
| `character-def.md` | 本 Def 的 `victory_conditions.ProtectTarget` 引用的 CharacterDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 遭遇战配置定义——描述一个战斗遭遇的完整配置。
///
/// EncounterDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<EncounterDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct EncounterDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（EncounterDef 前缀: `enc_`）
    pub id: EncounterId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 怪物编队 ──
    /// 遭遇战的生成组配置（引用 SpawnGroupDef）
    pub groups: Vec<EncounterGroup>,

    // ── 触发条件 ──
    /// 触发遭遇战的条件（引用 L1 ConditionDef，可选）
    ///
    /// 不设置 = 手动触发（如玩家点击 BOSS）；设置 = 自动触发（进入区域/事件触发）
    pub trigger_conditions: Option<Vec<ConditionId>>,

    // ── 胜负条件 ──
    /// 胜利条件（满足任一即胜利）
    pub victory_conditions: Vec<VictoryCondition>,
    /// 失败条件（满足任一即失败）
    pub defeat_conditions: Vec<DefeatCondition>,

    // ── 先攻与轮次 ──
    /// 先攻检定修正（影响 Encounter 中所有怪物的先攻值）
    pub initiative_bonus: Option<i32>,
    /// 遭遇战最大回合数（超过此回合数 = 自动失败，可选）
    pub max_rounds: Option<u32>,

    // ── 难度覆盖 ──
    /// 覆盖全局难度的特定难度配置（引用 L3 DifficultyDef，可选）
    pub difficulty_override: Option<DifficultyId>,

    // ── 战利品 ──
    /// 额外战利品表（战斗结束后额外掉落，与怪物自身掉落叠加）
    pub loot_bonus: Option<LootTableId>,
    /// 经验倍率（用于调整该遭遇的总经验）
    pub xp_multiplier: f32,

    // ── AI 配置 ──
    /// AI 行为覆盖（可选，覆盖怪物自身的 AIBehaviorHints）
    pub ai_config: Option<EncounterAIConfig>,

    // ── 环境 ──
    /// 环境标签（引用 L0 TagDef，如 "tag:dungeon", "tag:outdoor"）
    pub environment_tags: Vec<TagId>,

    // ── 元数据 ──
    /// 标签列表
    pub tags: Vec<TagId>,
}
```

### 内嵌数据结构

```rust
/// 遭遇战生成组——引用 SpawnGroupDef 并配置生成细节
#[derive(Deserialize, Clone, Debug)]
pub struct EncounterGroup {
    /// 引用的 SpawnGroupDef ID
    pub spawn_group_id: SpawnGroupId,
    /// 该生成组的实例数量（重复使用同一模板多次）
    pub count: u32,
    /// 生成位置提示（L4 软引用，由 MapDef 精确映射）
    pub position_hint: SpawnPosition,
    /// 延迟回合数（0 = 开局即出现，N = 第 N 回合作为增援出现）
    pub delay_rounds: Option<u32>,
    /// 触发增援的条件（引用 L1 ConditionDef，满足时 spawn_group 加入战斗）
    pub reinforcement_condition: Option<ConditionId>,
}

/// 生成位置提示——不定义精确坐标，仅给出位置倾向
#[derive(Deserialize, Clone, Debug)]
pub enum SpawnPosition {
    /// 随机位置
    Random,
    /// 固定网格坐标（由 L4 MapDef 解析——EncounterDef 不直接绑定地图）
    Fixed(u32, u32),
    /// 地图边缘随机
    Edge,
    /// 环绕目标
    Surround,
    /// 侧翼
    Flanking,
    /// 密集区域
    Clustered,
    /// 自定义位置（L4 软引用 key）
    Custom(String),
}

/// 胜利条件枚举
#[derive(Deserialize, Clone, Debug)]
pub enum VictoryCondition {
    /// 消灭所有敌人
    EliminateAll,
    /// 消灭指定生成组
    EliminateGroup(SpawnGroupId),
    /// 存活指定回合数
    SurviveRounds(u32),
    /// 保护指定角色存活（引用 CharacterDef）
    ProtectTarget(CharacterId),
    /// 击杀指定怪物类型（引用 MonsterDef）
    BossKill(MonsterId),
    /// 到达指定位置（L4 软引用）
    ReachPosition(String),
    /// 自定义条件（引用 L1 ConditionDef）
    Custom(ConditionId),
}

/// 失败条件枚举
#[derive(Deserialize, Clone, Debug)]
pub enum DefeatCondition {
    /// 所有玩家角色倒下
    AllPartyDown,
    /// 队伍全灭
    PartyWipe,
    /// 回合耗尽
    TimerExpired,
    /// 被保护目标倒下
    ProtectTargetDown(CharacterId),
    /// 自定义条件
    Custom(ConditionId),
}

/// 遭遇战级别 AI 配置覆盖
#[derive(Deserialize, Clone, Debug)]
pub struct EncounterAIConfig {
    /// AI 协作策略
    pub cooperation: AICooperation,
    /// 是否启用 Boss 阶段切换
    pub boss_phases: bool,
    /// 阶段切换的条件和效果（Boss 战专用）
    pub phase_transitions: Option<Vec<PhaseTransition>>,
}

/// AI 协作策略
#[derive(Deserialize, Clone, Debug)]
pub enum AICooperation {
    /// 各自为战
    Independent,
    /// 协作：集中攻击同一目标
    FocusFire,
    /// 协作：分兵保护/治疗
    Protect,
    /// 协作：包围
    FlankingManeuver,
    /// 脚本化行动模式
    Scripted,
    /// 无 AI（不行动）
    Passive,
}

/// Boss 阶段切换
#[derive(Deserialize, Clone, Debug)]
pub struct PhaseTransition {
    /// 阶段编号（从 1 开始）
    pub phase: u32,
    /// 进入该阶段的条件（引用 ConditionDef）
    pub trigger: ConditionId,
    /// 阶段性能力变化（引用 AbilityId，如解锁新技能）
    pub unlock_abilities: Option<Vec<AbilityId>>,
    /// 阶段性 Buff（引用 BuffId）
    pub grant_buffs: Option<Vec<BuffId>>,
    /// 是否恢复全部 HP
    pub full_heal: bool,
}
```

### 字段说明

- **`groups`**: 遭遇战可以包含多个生成组，每个是 SpawnGroupDef 的一次实例化。如"2 队哥布林弓箭手 + 1 个哥布林萨满"
- **`delay_rounds` / `reinforcement_condition`**: 增援机制——要么固定回合后出现，要么条件满足时出现。两者可同时设置（延迟 N 回合且条件满足时出）
- **`initiative_bonus`**: 遭遇战级别的先攻修正。如"该 Boss 战全体敌人先攻 +5"
- **`ai_config.phase_transitions`**: Boss 战专用，定义多阶段战斗。阶段 1 → 条件触发 → 阶段 2（清空 Debuff、恢复 HP、解锁新技能）

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// EncounterDef 注册插件
pub struct EncounterDefPlugin;

impl Plugin for EncounterDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<EncounterDef>();
        app.init_asset_loader::<RonAssetLoader<EncounterDef>>();
        app.insert_resource(DefRegistry::<EncounterDef>::new());
        app.add_systems(
            PreUpdate,
            load_encounter_defs
                .run_if(resource_changed::<Assets<EncounterDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按环境标签过滤 EncounterDef
pub fn get_encounters_by_environment(
    tag_id: &TagId,
    registry: &DefRegistry<EncounterDef>,
) -> Vec<&EncounterDef> {
    registry.iter()
        .filter(|def| def.environment_tags.iter().any(|t| t == tag_id))
        .collect()
}
```

### 注册生命周期

```
EncounterDefPlugin::build
  │
  ├── EncounterDef 从 assets/config/03_gameplay/encounters.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId) 引用存在性
        ├── L1 (ConditionId, AbilityId, BuffId) 引用存在性
        ├── L2 (CharacterId, MonsterId 间接) 引用存在性
        ├── L3 (SpawnGroupId, DifficultyId, LootTableId) 引用存在性
        ├── groups 非空
        ├── victory_conditions 非空
        ├── defeat_conditions 非空
        ├── xp_multiplier 范围（0.0-10.0）
        ├── L4 禁止引用检查
        └── 胜负条件可达性检查（避免不可能胜利/失败的条件组合）
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | EncounterId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `groups` 非空 | 遭遇战必须有至少一个生成组 |
| V4 | `victory_conditions` 非空 | 遭遇战必须有至少一个胜利条件 |
| V5 | `defeat_conditions` 非空 | 遭遇战必须有至少一个失败条件 |
| V6 | `xp_multiplier` 范围 | 0.0-10.0（默认 1.0） |
| V7 | `max_rounds` >= 1（如果设置） | 回合上限至少为 1 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `groups` 中的每个 SpawnGroupId 已注册 | 在 DefRegistry<SpawnGroupDef> 中存在 |
| V9 | `trigger_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V10 | `difficulty_override`（如果设置）已注册 | 在 DefRegistry<DifficultyDef> 中存在 |
| V11 | `loot_bonus`（如果设置）已注册 | 在 DefRegistry<LootTableDef> 中存在 |
| V12 | `victory_conditions` 中的 ConditionId (Custom) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V13 | `victory_conditions` 中的 CharacterId (ProtectTarget) 已注册 | 在 DefRegistry<CharacterDef> 中存在 |
| V14 | `defeat_conditions` 中的 CharacterId (ProtectTargetDown) 已注册 | 在 DefRegistry<CharacterDef> 中存在 |
| V15 | `defeat_conditions` 中的 ConditionId (Custom) 已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V16 | `ai_config.phase_transitions` 中的每个 AbilityId/BuffId 已注册 | 在对应 DefRegistry 中存在 |
| V17 | `environment_tags` 和 `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | EncounterDef 不得引用任何 L4 World Def | 层间依赖方向规则 |
| V19 | `SpawnPosition::Fixed(u32, u32)` 坐标由 L4 MapDef 解释，EncounterDef 不验证 | 坐标是相对位置提示，非绝对地图坐标 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V20 | 胜利/失败条件不可同时满足 | 如 EliminateAll 和 PartyWipe 不应在同一 Encounter 中同时可能 |
| V21 | 增援条件合理性 | `delay_rounds` 不应超过 `max_rounds`（如果设置） |
| V22 | Phase 编号连续性 | phase_transitions 中的 phase 编号应从 1 开始递增 |
| V23 | Boss 战应有 BossKill 胜利条件 | 含 phase_transitions 的 Encounter 建议使用 BossKill 条件 |

---

## 5. RON 示例

```ron
(
    id: "enc:forest_ambush",
    name_key: "encounter.enc_forest_ambush.name",
    description_key: "encounter.enc_forest_ambush.desc",
    schema_version: 1,

    groups: [
        (
            spawn_group_id: "spawn:goblin_ambushers",
            count: 2,
            position_hint: Flanking,
            delay_rounds: None,
            reinforcement_condition: None,
        ),
        (
            spawn_group_id: "spawn:goblin_shaman",
            count: 1,
            position_hint: Clustered,
            delay_rounds: Some(2),
            reinforcement_condition: Some("cond:if_any_goblin_dies"),
        ),
    ],

    trigger_conditions: Some([
        "cond:party_enters_forest_ambush_zone",
    ]),

    victory_conditions: [
        EliminateAll,
    ],

    defeat_conditions: [
        PartyWipe,
    ],

    initiative_bonus: Some(2),
    max_rounds: None,

    difficulty_override: None,

    loot_bonus: Some("loot:goblin_ambush_bonus"),
    xp_multiplier: 1.0,

    ai_config: Some((
        cooperation: FocusFire,
        boss_phases: false,
        phase_transitions: None,
    )),

    environment_tags: ["tag:forest", "tag:outdoor"],
    tags: ["tag:low_level", "tag:ambush"],
)
```

```ron
(
    id: "enc:dragon_peak_boss",
    name_key: "encounter.enc_dragon_peak_boss.name",
    description_key: "encounter.enc_dragon_peak_boss.desc",
    schema_version: 1,

    groups: [
        (
            spawn_group_id: "spawn:dragon_elder",
            count: 1,
            position_hint: Custom("loc:boss_arena_center"),
            delay_rounds: None,
            reinforcement_condition: None,
        ),
        (
            spawn_group_id: "spawn:dragon_cultists",
            count: 2,
            position_hint: Edge,
            delay_rounds: Some(3),
            reinforcement_condition: None,
        ),
    ],

    trigger_conditions: Some([
        "cond:party_enters_dragon_throne_room",
    ]),

    victory_conditions: [
        BossKill("mob:dragon_elder"),
    ],

    defeat_conditions: [
        PartyWipe,
        ProtectTargetDown("chr:king_arthur"),
    ],

    initiative_bonus: Some(5),
    max_rounds: Some(30),

    difficulty_override: Some("diff:hard"),

    loot_bonus: Some("loot:dragon_peak_clear_reward"),
    xp_multiplier: 1.5,

    ai_config: Some((
        cooperation: Scripted,
        boss_phases: true,
        phase_transitions: Some([
            (
                phase: 2,
                trigger: "cond:dragon_hp_below_50_pct",
                unlock_abilities: Some(["ability:dragon_fury_breath"]),
                grant_buffs: Some(["buff:dragon_enrage"]),
                full_heal: false,
            ),
            (
                phase: 3,
                trigger: "cond:dragon_hp_below_20_pct",
                unlock_abilities: Some(["ability:dragon_desolation"]),
                grant_buffs: None,
                full_heal: true,
            ),
        ]),
    )),

    environment_tags: ["tag:dungeon", "tag:boss_arena"],
    tags: ["tag:boss", "tag:high_level", "tag:dragon"],
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | EncounterDef 的关系 |
|--------|--------------------|
| SpawnGroupDef | EncounterDef 通过 `groups[].spawn_group_id` 引用 SpawnGroupDef。一个 SpawnGroupDef 可被多个 EncounterDef 引用 |
| DifficultyDef | EncounterDef 通过 `difficulty_override` 可选覆盖全局难度 |
| LootTableDef | EncounterDef 通过 `loot_bonus` 引用额外掉落表 |
| QuestDef | EncounterDef 不直接引用 QuestDef。L4 SceneDef 或 Quest 领域监听 Encounter 完成事件来推进任务进度 |

**重要**：EncounterDef 不包含地图位置。遭遇战具体出现在哪张地图上由 L4 MapDef 定义。L4 MapDef 引用 EncounterDef，是反向关联：

```
MapDef (L4)
  └── encounter_spawns: [
        (encounter_id: "enc:forest_ambush", position: (23, 45)),
        (encounter_id: "enc:dragon_peak_boss", position: (120, 80)),
      ]
```

---

*本文档由 @content-architect 维护。*
