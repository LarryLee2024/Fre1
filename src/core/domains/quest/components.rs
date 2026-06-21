//! ECS Components — 任务领域组件与类型
//!
//! 定义任务相关的 ID 类型、值类型、ECS 组件。
//! 详见 docs/02-domain/domains/quest_domain.md
//! 详见 docs/04-data/domains/quest_schema.md

use bevy::asset::Asset;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── ID 类型 ──────────────────────────────────────────────────────

/// 任务定义标识符（前缀: `qst_`）。
///
/// 统一使用 shared::ids::QuestId。
pub use crate::shared::ids::QuestId as QuestDefId;
use crate::shared::localization_key::LocalizationKey;

/// 目标唯一标识符（任务内唯一）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct ObjectiveId(pub String);

// ─── 值类型 ────────────────────────────────────────────────────────

/// 任务生命周期状态机。
///
/// 状态转换图（线性推进 + 异常终态）：
/// ```text
/// Unavailable → Available → Active → Completed
///                     │                  │
///                     └──→ Failed ←──────┘
/// ```
/// - Unavailable: 前置条件未满足
/// - Available: 可接取
/// - Active: 进行中
/// - Completed: 所有目标完成
/// - Failed: 任务失败
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum QuestState {
    /// 前置条件未满足，不可接取。
    Unavailable,
    /// 可接取。
    Available,
    /// 进行中。
    Active,
    /// 已完成。
    Completed,
    /// 已失败。
    Failed,
}

/// 任务类型。
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum QuestType {
    /// 主线任务。
    Main,
    /// 支线任务。
    Side,
    /// 阵营任务。
    Faction,
    /// 同伴任务。
    Companion,
    /// 世界事件。
    World,
}

/// 目标类型。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ObjectiveType {
    /// 击杀特定类型敌人。
    Kill { enemy_tags: Vec<String> },
    /// 收集特定物品。
    Collect { item_ids: Vec<String> },
    /// 与 NPC 对话。
    Talk { npc_id: String },
    /// 到达特定位置。
    Reach { area_id: String },
    /// 护送目标到目的地。
    Escort {
        target_id: String,
        destination: String,
    },
    /// 使用物品。
    Use {
        item_id: String,
        target_id: Option<String>,
    },
    /// 自定义条件。
    Custom,
}

/// 奖励解锁类型。
#[derive(Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum UnlockType {
    /// 解锁新任务。
    Quest,
    /// 解锁新区域。
    Area,
    /// 解锁新能力。
    Ability,
    /// 解锁新配方。
    Recipe,
}

// ─── Definition 层结构 ──────────────────────────────────────────

/// 任务前置条件。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum PrereqType {
    /// 最低等级要求。
    Level { min_level: u32 },
    /// 前置任务必须完成。
    QuestCompleted { quest_id: QuestDefId },
    /// 阵营声望要求。
    Reputation { faction_id: String, min_level: u32 },
    /// StoryFlag 检查。
    StoryFlag { flag_id: String, value: String },
}

/// 任务前置条件。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct QuestPrereq {
    pub prereq_type: PrereqType,
    /// 直接指定的前置任务。
    pub required_quest: Option<QuestDefId>,
}

/// 目标定义。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ObjectiveDef {
    /// 目标 ID。
    pub id: ObjectiveId,
    /// 描述本地化 Key。
    pub description_key: String,
    /// 目标类型。
    pub objective_type: ObjectiveType,
    /// 目标值（如"击杀 5 只"中的 5）。
    pub target_value: u32,
    /// 关联 ID。
    pub associated_id: Option<String>,
}

/// 任务奖励定义。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct QuestRewardDef {
    /// 经验奖励。
    pub xp_reward: u64,
    /// 金币奖励。
    pub gold_reward: u64,
    /// 物品奖励列表。
    pub item_rewards: Vec<ItemReward>,
    /// 声望奖励列表。
    pub reputation_rewards: Vec<ReputationReward>,
    /// 解锁奖励列表。
    pub unlocks: Vec<UnlockReward>,
}

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ItemReward {
    pub item_id: String,
    pub quantity: u32,
}

/// 任务完成奖励：声望变更。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ReputationReward {
    /// 受影响的阵营 ID
    pub faction_id: String,
    /// 声望变化量（正数增加，负数减少）
    pub amount: i32,
}

/// 任务完成奖励：解锁内容。
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct UnlockReward {
    /// 解锁类型（天赋/法术/配方等）
    pub unlock_type: UnlockType,
    /// 被解锁内容的 ID
    pub unlock_id: String,
}

/// 任务静态定义（Definition 层）。
#[derive(Debug, Clone, Asset, Serialize, Deserialize, Reflect)]
pub struct QuestDef {
    pub id: QuestDefId,
    #[reflect(ignore)]
    pub name_key: LocalizationKey,
    #[reflect(ignore)]
    pub desc_key: LocalizationKey,
    pub quest_type: QuestType,
    pub prerequisites: Vec<QuestPrereq>,
    pub objectives: Vec<ObjectiveDef>,
    pub rewards: QuestRewardDef,
    pub is_critical: bool,
    pub exclusive_with: Vec<QuestDefId>,
}

// ─── Instance 层组件 ─────────────────────────────────────────────

/// 目标运行时进度。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ObjectiveProgress {
    /// 目标 ID。
    pub objective_id: ObjectiveId,
    /// 当前进度值。
    pub current_value: u32,
    /// 目标值。
    pub target_value: u32,
    /// 是否已完成。
    pub is_completed: bool,
}

impl ObjectiveProgress {
    pub fn new(objective_id: ObjectiveId, target_value: u32) -> Self {
        Self {
            objective_id,
            current_value: 0,
            target_value,
            is_completed: false,
        }
    }

    /// 增加进度，返回是否刚好在此次调用时完成。
    pub fn advance(&mut self, amount: u32) -> bool {
        self.current_value = self.current_value.saturating_add(amount);
        if !self.is_completed && self.current_value >= self.target_value {
            self.is_completed = true;
            self.current_value = self.target_value;
            true
        } else {
            false
        }
    }
}

/// 任务日志条目。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct QuestEntry {
    /// 任务 ID。
    pub quest_id: QuestDefId,
    /// 当前任务状态。
    pub state: QuestState,
    /// 各目标进度。
    pub objective_progress: Vec<ObjectiveProgress>,
    /// 失败原因。
    pub fail_reason: Option<String>,
}

impl QuestEntry {
    pub fn new(quest_id: QuestDefId, objectives: Vec<ObjectiveDef>) -> Self {
        let objective_progress = objectives
            .into_iter()
            .map(|obj| ObjectiveProgress::new(obj.id, obj.target_value))
            .collect();
        Self {
            quest_id,
            state: QuestState::Unavailable,
            objective_progress,
            fail_reason: None,
        }
    }

    /// 所有目标是否已完成。
    pub fn all_objectives_completed(&self) -> bool {
        self.objective_progress.iter().all(|p| p.is_completed)
    }

    /// 是否奖励已发放（任务为 Completed 已完成状态即视为已发放）。
    pub fn is_reward_granted(&self) -> bool {
        self.state == QuestState::Completed
    }
}

/// 任务日志组件。
///
/// 记录队伍/玩家的所有任务追踪状态。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct QuestLog {
    /// 所有任务的当前状态。
    pub entries: Vec<QuestEntry>,
    /// 已完成任务总数。
    pub completed_count: u32,
}

impl QuestLog {
    /// 创建空任务日志。无活跃任务，后续通过 add_entry() 添加。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            completed_count: 0,
        }
    }

    /// 按任务 ID 查询任务条目（只读）。
    pub fn get_entry(&self, quest_id: &QuestDefId) -> Option<&QuestEntry> {
        self.entries.iter().find(|e| e.quest_id == *quest_id)
    }

    /// 按任务 ID 查询任务条目（可变引用，用于更新进度）。
    pub fn get_entry_mut(&mut self, quest_id: &QuestDefId) -> Option<&mut QuestEntry> {
        self.entries.iter_mut().find(|e| e.quest_id == *quest_id)
    }
}

impl Default for QuestLog {
    fn default() -> Self {
        Self::new()
    }
}
