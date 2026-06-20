//! 领域错误 — Quest 域程序错误枚举。
//!
//! 涵盖任务系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `QuestFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

use super::components::QuestDefId;

/// 任务系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"前置条件未满足"）请使用 [`QuestFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum QuestError {
    /// 任务未找到。
    #[error("quest not found: {quest_id}")]
    QuestNotFound { quest_id: QuestDefId },
    /// 目标未找到。
    #[error("objective not found: {objective_id}")]
    ObjectiveNotFound { objective_id: String },
}
