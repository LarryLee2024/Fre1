//! quest_logger — 任务事件日志 Observer
//!
//! 监听任务生命周期事件（接受/目标完成/交付/失败/进度更新），生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::domains::quest::events::{
    ObjectiveCompleted, QuestAccepted, QuestFailed, QuestProgressUpdated, QuestTurnedIn,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 任务接受日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::QST001, event = "任务接受"))]
pub(crate) fn on_quest_accepted(trigger: On<QuestAccepted>) {
    metrics::record(LogCode::QST001);
    let event = trigger.event();
    info!(
        code = ?LogCode::QST001,
        event = "任务接受",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        "任务接受"
    );
}

/// 目标完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::QST002, event = "目标完成"))]
pub(crate) fn on_objective_completed(trigger: On<ObjectiveCompleted>) {
    metrics::record(LogCode::QST002);
    let event = trigger.event();
    info!(
        code = ?LogCode::QST002,
        event = "目标完成",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        objective_id = %event.objective_id,
        "目标完成"
    );
}

/// 任务交付日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::QST003, event = "任务交付"))]
pub(crate) fn on_quest_turned_in(trigger: On<QuestTurnedIn>) {
    metrics::record(LogCode::QST003);
    let event = trigger.event();
    info!(
        code = ?LogCode::QST003,
        event = "任务交付",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        "任务交付"
    );
}

/// 任务失败日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::QST004, event = "任务失败"))]
pub(crate) fn on_quest_failed(trigger: On<QuestFailed>) {
    metrics::record(LogCode::QST004);
    let event = trigger.event();
    warn!(
        code = ?LogCode::QST004,
        event = "任务失败",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        reason = %event.fail_reason,
        "任务失败"
    );
}

/// 任务进度更新日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::QST005, event = "任务进度更新"))]
pub(crate) fn on_quest_progress_updated(trigger: On<QuestProgressUpdated>) {
    metrics::record(LogCode::QST005);
    let event = trigger.event();
    info!(
        code = ?LogCode::QST005,
        event = "任务进度更新",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        objective_id = %event.objective_id,
        old = event.old_progress,
        new = event.new_progress,
        target = event.target,
        "任务进度更新"
    );
}
