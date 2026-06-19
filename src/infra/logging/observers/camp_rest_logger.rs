//! camp_rest_logger — CampRest 域日志 Observer
//!
//! 监听营地休息事件（短休/长休/中断），生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::camp_rest::events::{
    CampEventTriggered, LongRestCompleted, LongRestInterrupted, LongRestStarted, ShortRestCompleted,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 短休完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CNR001, event = "short_rest_completed"))]
pub(crate) fn on_short_rest_completed(trigger: On<ShortRestCompleted>) {
    metrics::record(LogCode::CNR001);
    let event = trigger.event();
    info!(
        code = ?LogCode::CNR001,
        event = "short_rest_completed",
        entities = ?event.entities,
        hit_dice_used = event.hit_dice_used,
        hp_healed = event.hp_healed,
        "short_rest_completed"
    );
}

/// 长休开始日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CNR002, event = "long_rest_started"))]
pub(crate) fn on_long_rest_started(trigger: On<LongRestStarted>) {
    metrics::record(LogCode::CNR002);
    let event = trigger.event();
    info!(
        code = ?LogCode::CNR002,
        event = "long_rest_started",
        entities = ?event.entities,
        location = %event.camp_location,
        "long_rest_started"
    );
}

/// 长休完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CNR003, event = "long_rest_completed"))]
pub(crate) fn on_long_rest_completed(trigger: On<LongRestCompleted>) {
    metrics::record(LogCode::CNR003);
    let event = trigger.event();
    info!(
        code = ?LogCode::CNR003,
        event = "long_rest_completed",
        hp_restored = event.hp_restored,
        slots_restored = event.spell_slots_restored,
        "long_rest_completed"
    );
}

/// 长休中断日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CNR004, event = "long_rest_interrupted"))]
pub(crate) fn on_long_rest_interrupted(trigger: On<LongRestInterrupted>) {
    metrics::record(LogCode::CNR004);
    let event = trigger.event();
    warn!(
        code = ?LogCode::CNR004,
        event = "long_rest_interrupted",
        entities = ?event.entities,
        source = %event.interruption_source,
        "long_rest_interrupted"
    );
}

/// 营地事件触发日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CNR005, event = "camp_event_triggered"))]
pub(crate) fn on_camp_event_triggered(trigger: On<CampEventTriggered>) {
    metrics::record(LogCode::CNR005);
    let event = trigger.event();
    info!(
        code = ?LogCode::CNR005,
        event = "camp_event_triggered",
        event_type = %event.event_type,
        participants = ?event.participants,
        "camp_event_triggered"
    );
}
