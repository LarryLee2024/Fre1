---
id: domains.quest.schema.v1
title: Quest Schema — 任务数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: false
---

# Quest Schema — 任务数据架构

> **领域归属**: Domains — 叙事内容层 | **依赖 Schema**: Event, Condition, Faction, Progression, Inventory | **定义依据**: `docs/02-domain/domains/quest_domain.md`

---

## 1. Schema Design

### 1.1 QuestDef（Definition 层）

```rust
/// 任务模板定义。内容团队配置，运行时只读。
struct QuestDef {
    /// 任务唯一标识（前缀: `qst_`）
    id: QuestDefId,

    /// 任务名称本地化 Key
    name_key: LocalizationKey,

    /// 任务描述本地化 Key
    desc_key: LocalizationKey,

    /// 任务类型
    quest_type: QuestType,

    /// 前置条件（全部满足才可接取）
    prerequisites: Vec<QuestPrereq>,

    /// 任务目标列表
    objectives: Vec<ObjectiveDef>,

    /// 任务失败条件
    fail_conditions: Vec<FailCondition>,

    /// 奖励定义
    rewards: QuestRewardDef,

    /// 是否为关键/主线任务（不可放弃）
    is_critical: bool,

    /// 互斥任务列表（接了此任务后不可接互斥任务）
    exclusive_with: Vec<QuestDefId>,
}

enum QuestType { Main, Side, Faction, Companion, World }

struct QuestPrereq {
    /// 前置条件类型
    prereq_type: PrereqType,
    /// 条件 ID 或直接值
    condition: Option<ConditionDefId>,
    /// 直接指定的前置任务
    required_quest: Option<QuestDefId>,
    /// 前置任务状态要求
    required_state: QuestState,
}

enum PrereqType {
    Level { min_level: u32 },
    QuestCompleted { quest_id: QuestDefId },
    Reputation { faction_id: FactionDefId, min_level: ReputationLevel },
    StoryFlag { flag_id: StoryFlagId, value: String },
    Condition { condition_id: ConditionDefId },
}

enum QuestState { Unavailable, Available, Active, Completed, Failed }
```

### 1.2 ObjectiveDef（Definition 层）

```rust
/// 任务目标的静态定义。
struct ObjectiveDef {
    /// 目标唯一标识（任务内唯一）
    id: ObjectiveId,

    /// 目标描述本地化 Key
    description_key: LocalizationKey,

    /// 目标类型
    objective_type: ObjectiveType,

    /// 目标值（如"击杀 5 只"中的 5）
    target_value: u32,

    /// 关联 ID（要击杀的敌人 Tag/要收集的物品 ID/要对话的 NPC）
    associated_id: Option<String>,

    /// 可选条件
    condition: Option<ConditionDefId>,
}

enum ObjectiveType {
    Kill { enemy_tags: Vec<TagDefId> },
    Collect { item_ids: Vec<ItemDefId> },
    Talk { npc_id: EntityId },
    Reach { position: GridPosition, area_id: String },
    Escort { target_id: EntityId, destination: GridPosition },
    Use { item_id: ItemDefId, target_id: Option<EntityId> },
    Custom,
}
```

### 1.3 ObjectiveProgress（Instance 层/Persistence 层）

```rust
/// 任务目标的运行时进度。
struct ObjectiveProgress {
    /// 目标 ID
    objective_id: ObjectiveId,

    /// 当前进度值
    current_value: u32,

    /// 目标值
    target_value: u32,

    /// 是否已完成
    is_completed: bool,
}
```

### 1.4 QuestRewardDef（Definition 层）

```rust
/// 任务奖励定义。
struct QuestRewardDef {
    /// 经验奖励
    xp_reward: u64,

    /// 物品奖励
    item_rewards: Vec<ItemReward>,

    /// 声望奖励
    reputation_rewards: Vec<ReputationReward>,

    /// 解锁奖励（新任务/新区域/新能力）
    unlocks: Vec<UnlockReward>,

    /// 金币奖励
    gold_reward: u64,
}

struct ItemReward { item_id: ItemDefId, quantity: u32 }

struct ReputationReward { faction_id: FactionDefId, amount: i32 }

struct UnlockReward {
    unlock_type: UnlockType,
    unlock_id: String,
}

enum UnlockType { Quest, Area, Ability, Recipe }
```

### 1.5 QuestLog（Instance 层/Persistence 层）

```rust
/// 任务日志——玩家的任务追踪状态。
struct QuestLog {
    /// 所有任务的当前状态
    entries: Vec<QuestEntry>,

    /// 已完成任务总数（用于 UI 统计）
    completed_count: u32,
}

struct QuestEntry {
    /// 任务 ID
    quest_id: QuestDefId,

    /// 当前任务状态
    state: QuestState,

    /// 各目标进度
    objective_progress: Vec<ObjectiveProgress>,

    /// 任务失败原因（如已失败）
    fail_reason: Option<String>,

    /// 接受任务的时间戳
    accepted_time: GameTime,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `QuestDef`, `ObjectiveDef`, `QuestRewardDef` | 任务、目标、奖励的静态配置 |
| **Spec** | — | Quest 无 Spec 层 |
| **Instance** | `QuestLog`, `ObjectiveProgress` | 任务运行时进度追踪 |
| **Persistence** | `QuestLog` | 任务状态和目标进度完整持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → ConditionSchema | 任务前置条件和目标条件 |
| → EventSchema | 任务事件发布（QuestAccepted, QuestTurnedIn 等） |
| → FactionSchema | 声望前置条件和奖励 |
| → ProgressionSchema | 经验奖励 |
| → InventorySchema | 物品奖励 |
| ← NarrativeSchema | 对话可触发任务接受/推进 |
| ← CombatSchema | 击杀事件用于更新 Kill 类目标 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — 任务是玩家进程数据

### Save

- `QuestLog` 完整持久化（所有任务状态 + 目标进度）
- QuestDef/ObjectiveDef/QuestRewardDef 从配置加载
- 奖励一次性发放，存档标记已发放（防止读档后重复领取）

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 前置链完整 | 所有 prerequisites 满足后才可接受 | 拒绝接受 |
| 进度只增不减 | current_value 不可减少 | 运行时断言 |
| 奖励不重复 | 每个任务只发一次奖励 | 存档标记检查 |
| 任务互斥 | exclusive_with 任务不可同时 Active | 接受时检查 |
| 关键任务保护 | is_critical 任务不可放弃/失败 | 运行时断言 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: QuestDef/ObjectiveDef 为 Definition，QuestLog/ObjectiveProgress 为 Instance
- ✅ **Data Law 002 (Rule-Content分离)**: 任务条件为内容配置，进度更新逻辑为代码
- ✅ **Data Law 003 (配置只引用ID)**: QuestDef 引用 ConditionDefId/FactionDefId/ItemDefId
- ✅ **Data Law 011 (Schema版本化)**: QuestLog 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Quest 通过 Event 与 Combat/Narrative/Inventory 通信，不直接修改外部数据
