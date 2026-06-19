//! quest_logger — 任务事件日志 Observer
//!
//! 监听任务生命周期事件（接受/目标完成/交付/失败/进度更新），生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::domains::quest::events::{
    ObjectiveCompleted, QuestAccepted, QuestFailed, QuestProgressUpdated, QuestTurnedIn,
};
use crate::shared::diagnostics::LogCode;

/// 任务接受日志 Observer。
pub(crate) fn on_quest_accepted(trigger: On<QuestAccepted>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::QST001,
        event = "quest_accepted",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        "quest_accepted"
    );
}

/// 目标完成日志 Observer。
pub(crate) fn on_objective_completed(trigger: On<ObjectiveCompleted>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::QST002,
        event = "objective_completed",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        objective_id = %event.objective_id,
        "objective_completed"
    );
}

/// 任务交付日志 Observer。
pub(crate) fn on_quest_turned_in(trigger: On<QuestTurnedIn>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::QST003,
        event = "quest_turned_in",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        "quest_turned_in"
    );
}

/// 任务失败日志 Observer。
pub(crate) fn on_quest_failed(trigger: On<QuestFailed>) {
    let event = trigger.event();
    warn!(
        code = ?LogCode::QST004,
        event = "quest_failed",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        reason = %event.fail_reason,
        "quest_failed"
    );
}

/// 任务进度更新日志 Observer。
pub(crate) fn on_quest_progress_updated(trigger: On<QuestProgressUpdated>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::QST005,
        event = "quest_progress_updated",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        objective_id = %event.objective_id,
        old = event.old_progress,
        new = event.new_progress,
        target = event.target,
        "quest_progress_updated"
    );
}
