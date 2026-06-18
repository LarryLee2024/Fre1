//! 领域错误 — Quest 域错误枚举
//!
//! 涵盖任务接取、进度更新、奖励发放等操作的错误。
//! 详见 docs/02-domain/domains/quest_domain.md §4

use bevy::prelude::*;

use super::components::QuestDefId;

/// 任务系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum QuestError {
    /// 任务未找到。
    QuestNotFound { quest_id: QuestDefId },
    /// 前置条件未满足。
    PrerequisitesNotMet {
        quest_id: QuestDefId,
        reason: String,
    },
    /// 任务已处于不可接取状态。
    NotAvailable {
        quest_id: QuestDefId,
        current_state: String,
    },
    /// 任务已完成（不可重新接受）。
    AlreadyCompleted { quest_id: QuestDefId },
    /// 任务奖励已发放。
    RewardAlreadyGranted { quest_id: QuestDefId },
    /// 互斥任务当前激活中。
    ExclusiveQuestActive {
        quest_id: QuestDefId,
        exclusive_with: QuestDefId,
    },
    /// 关键任务不可放弃。
    CriticalQuestCannotAbandon { quest_id: QuestDefId },
    /// 目标未找到。
    ObjectiveNotFound { objective_id: String },
}
