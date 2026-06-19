//! 领域错误 — Quest 域错误枚举
//!
//! 涵盖任务接取、进度更新、奖励发放等操作的错误。
//! 详见 docs/02-domain/domains/quest_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

use super::components::QuestDefId;

/// 任务系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum QuestError {
    /// 任务未找到。
    #[error("quest not found: {quest_id}")]
    QuestNotFound { quest_id: QuestDefId },
    /// 前置条件未满足。
    #[error("prerequisites not met for quest {quest_id}: {reason}")]
    PrerequisitesNotMet {
        quest_id: QuestDefId,
        reason: String,
    },
    /// 任务已处于不可接取状态。
    #[error("quest {quest_id} not available: current_state={current_state}")]
    NotAvailable {
        quest_id: QuestDefId,
        current_state: String,
    },
    /// 任务已完成（不可重新接受）。
    #[error("quest already completed: {quest_id}")]
    AlreadyCompleted { quest_id: QuestDefId },
    /// 任务奖励已发放。
    #[error("quest reward already granted: {quest_id}")]
    RewardAlreadyGranted { quest_id: QuestDefId },
    /// 互斥任务当前激活中。
    #[error("exclusive quest active: {quest_id} conflicts with {exclusive_with}")]
    ExclusiveQuestActive {
        quest_id: QuestDefId,
        exclusive_with: QuestDefId,
    },
    /// 关键任务不可放弃。
    #[error("critical quest cannot be abandoned: {quest_id}")]
    CriticalQuestCannotAbandon { quest_id: QuestDefId },
    /// 目标未找到。
    #[error("objective not found: {objective_id}")]
    ObjectiveNotFound { objective_id: String },
}
