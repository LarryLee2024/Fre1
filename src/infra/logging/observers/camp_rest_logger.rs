//! camp_rest_logger — CampRest 域日志 Observer
//!
//! 监听营地休息事件（短休/长休/中断），生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::camp_rest::events::{
    CampEventTriggered, LongRestCompleted, LongRestInterrupted, LongRestStarted, ShortRestCompleted,
};
use crate::shared::diagnostics::LogCode;
use crate::{emit_info, emit_warn};

/// 短休完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.camp_rest", fields(
    code = ?LogCode::CNR001,
    event = "short_rest_completed",
))]
pub(crate) fn on_short_rest_completed(trigger: On<ShortRestCompleted>) {
    let event = trigger.event();
    emit_info!(
        LogCode::CNR001,
        entities = ?event.entities,
        hit_dice_used = event.hit_dice_used,
        hp_healed = event.hp_healed,
        "短休完成",
    );
}

/// 长休开始日志 Observer。
#[tracing::instrument(skip_all, target = "domain.camp_rest", fields(
    code = ?LogCode::CNR002,
    event = "long_rest_started",
))]
pub(crate) fn on_long_rest_started(trigger: On<LongRestStarted>) {
    let event = trigger.event();
    emit_info!(
        LogCode::CNR002,
        entities = ?event.entities,
        location = %event.camp_location,
        "长休开始",
    );
}

/// 长休完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.camp_rest", fields(
    code = ?LogCode::CNR003,
    event = "long_rest_completed",
))]
pub(crate) fn on_long_rest_completed(trigger: On<LongRestCompleted>) {
    let event = trigger.event();
    emit_info!(
        LogCode::CNR003,
        hp_restored = event.hp_restored,
        slots_restored = event.spell_slots_restored,
        "长休完成",
    );
}

/// 长休中断日志 Observer。
#[tracing::instrument(skip_all, target = "domain.camp_rest", fields(
    code = ?LogCode::CNR004,
    event = "long_rest_interrupted",
))]
pub(crate) fn on_long_rest_interrupted(trigger: On<LongRestInterrupted>) {
    let event = trigger.event();
    emit_warn!(
        LogCode::CNR004,
        entities = ?event.entities,
        source = %event.interruption_source,
        "长休中断",
    );
}

/// 营地事件触发日志 Observer。
#[tracing::instrument(skip_all, target = "domain.camp_rest", fields(
    code = ?LogCode::CNR005,
    event = "camp_event_triggered",
))]
pub(crate) fn on_camp_event_triggered(trigger: On<CampEventTriggered>) {
    let event = trigger.event();
    emit_info!(
        LogCode::CNR005,
        event_type = %event.event_type,
        participants = ?event.participants,
        "营地事件触发",
    );
}
