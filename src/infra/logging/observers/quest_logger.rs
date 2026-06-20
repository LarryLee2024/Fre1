//! quest_logger — 任务事件日志 Observer
//!
//! 监听任务生命周期事件（接受/目标完成/交付/失败/进度更新），生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::quest::events::{
    ObjectiveCompleted, QuestAccepted, QuestFailed, QuestProgressUpdated, QuestTurnedIn,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 任务接受日志 Observer。
#[tracing::instrument(skip_all, target = "domain.quest", fields(
    code = ?LogCode::QST001,
    event = "quest_accepted",
))]
pub(crate) fn on_quest_accepted(trigger: On<QuestAccepted>) {
    telemetry::emit(LogCode::QST001);
    let event = trigger.event();
    info!(
        target = "domain.quest",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        "任务接受",
    );
}

/// 目标完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.quest", fields(
    code = ?LogCode::QST002,
    event = "quest_objective_completed",
))]
pub(crate) fn on_objective_completed(trigger: On<ObjectiveCompleted>) {
    telemetry::emit(LogCode::QST002);
    let event = trigger.event();
    info!(
        target = "domain.quest",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        objective_id = %event.objective_id,
        "目标完成",
    );
}

/// 任务交付日志 Observer。
#[tracing::instrument(skip_all, target = "domain.quest", fields(
    code = ?LogCode::QST003,
    event = "quest_completed",
))]
pub(crate) fn on_quest_turned_in(trigger: On<QuestTurnedIn>) {
    telemetry::emit(LogCode::QST003);
    let event = trigger.event();
    info!(
        target = "domain.quest",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        "任务交付",
    );
}

/// 任务失败日志 Observer。
#[tracing::instrument(skip_all, target = "domain.quest", fields(
    code = ?LogCode::QST004,
    event = "quest_failed",
))]
pub(crate) fn on_quest_failed(trigger: On<QuestFailed>) {
    telemetry::emit(LogCode::QST004);
    let event = trigger.event();
    warn!(
        target = "domain.quest",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        reason = %event.fail_reason,
        "任务失败",
    );
}

/// 任务进度更新日志 Observer。
#[tracing::instrument(skip_all, target = "domain.quest", fields(
    code = ?LogCode::QST005,
    event = "quest_progress_changed",
))]
pub(crate) fn on_quest_progress_updated(trigger: On<QuestProgressUpdated>) {
    telemetry::emit(LogCode::QST005);
    let event = trigger.event();
    info!(
        target = "domain.quest",
        entity = ?event.entity,
        quest_id = %event.quest_id,
        objective_id = %event.objective_id,
        old = event.old_progress,
        new = event.new_progress,
        target = event.target,
        "任务进度更新",
    );
}
