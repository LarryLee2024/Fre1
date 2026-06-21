//! 规则失败 — Quest 域业务规则不满足结果。
//!
//! 与 `QuestError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

use super::components::QuestDefId;

/// 任务系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum QuestFailure {
    /// 前置条件未满足。
    #[error("quest {quest_id} 前置条件不满足: {reason}")]
    PrerequisitesNotMet {
        quest_id: QuestDefId,
        reason: String,
    },
    /// 任务已处于不可接取状态。
    #[error("quest {quest_id} 不可接取: current_state={current_state}")]
    NotAvailable {
        quest_id: QuestDefId,
        current_state: String,
    },
    /// 任务已完成（不可重新接受）。
    #[error("quest 已完成: {quest_id}")]
    AlreadyCompleted { quest_id: QuestDefId },
    /// 任务奖励已发放。
    #[error("quest 奖励已发放: {quest_id}")]
    RewardAlreadyGranted { quest_id: QuestDefId },
    /// 互斥任务当前激活中。
    #[error("互斥 quest 激活中: {quest_id} 与 {exclusive_with} 冲突")]
    ExclusiveQuestActive {
        quest_id: QuestDefId,
        exclusive_with: QuestDefId,
    },
    /// 关键任务不可放弃。
    #[error("关键 quest 不可放弃: {quest_id}")]
    CriticalQuestCannotAbandon { quest_id: QuestDefId },
}

crate::impl_rule_failure!(QuestFailure,
    Self::PrerequisitesNotMet { .. } => "QUEST_PREREQUISITES_NOT_MET",
    Self::NotAvailable { .. } => "QUEST_NOT_AVAILABLE",
    Self::AlreadyCompleted { .. } => "QUEST_ALREADY_COMPLETED",
    Self::RewardAlreadyGranted { .. } => "QUEST_REWARD_ALREADY_GRANTED",
    Self::ExclusiveQuestActive { .. } => "QUEST_EXCLUSIVE_ACTIVE",
    Self::CriticalQuestCannotAbandon { .. } => "QUEST_CRITICAL_CANNOT_ABANDON",
);
