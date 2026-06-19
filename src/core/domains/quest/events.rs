//! 领域事件 — Quest 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event。
//! 事件订阅关系详见 docs/02-domain/domains/quest_domain.md §6

use bevy::prelude::*;

use super::components::{ObjectiveDef, QuestDefId, QuestRewardDef};

/// 任务接受事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct QuestAccepted {
    /// 接受者实体。
    pub entity: Entity,
    /// 任务 ID。
    pub quest_id: QuestDefId,
    /// 初始目标列表。
    pub objectives: Vec<ObjectiveDef>,
}

/// 目标完成事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct ObjectiveCompleted {
    /// 实体。
    pub entity: Entity,
    /// 任务 ID。
    pub quest_id: QuestDefId,
    /// 目标 ID。
    pub objective_id: String,
    /// 目标类型描述。
    pub objective_type: String,
}

/// 任务交付完成事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct QuestTurnedIn {
    /// 实体。
    pub entity: Entity,
    /// 任务 ID。
    pub quest_id: QuestDefId,
    /// 奖励定义。
    pub rewards: QuestRewardDef,
}

/// 任务失败事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct QuestFailed {
    /// 实体。
    pub entity: Entity,
    /// 任务 ID。
    pub quest_id: QuestDefId,
    /// 失败原因。
    pub fail_reason: String,
}

/// 任务进度更新事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct QuestProgressUpdated {
    /// 实体。
    pub entity: Entity,
    /// 任务 ID。
    pub quest_id: QuestDefId,
    /// 目标 ID。
    pub objective_id: String,
    /// 旧进度。
    pub old_progress: u32,
    /// 新进度。
    pub new_progress: u32,
    /// 目标值。
    pub target: u32,
}
