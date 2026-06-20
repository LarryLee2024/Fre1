//! effect_logger — 效果事件日志 Observer
//!
//! 监听效果生命周期事件（施加/移除/Tick/免疫），生成 INFO/WARN 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::capabilities::effect::events::{
    EffectApplied, EffectImmunityTriggered, EffectRemoved, EffectTicked,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 效果施加日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::EFF001, event = "效果施加"))]
pub(crate) fn on_effect_applied(trigger: On<EffectApplied>) {
    metrics::record(LogCode::EFF001);
    let event = trigger.event();
    info!(
        code = ?LogCode::EFF001,
        event = "效果施加",
        instance = %event.instance_id,
        def_id = %event.def_id,
        target = %event.target_entity,
        duration = %event.duration_type,
        "效果施加"
    );
}

/// 效果移除日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::EFF002, event = "效果移除"))]
pub(crate) fn on_effect_removed(trigger: On<EffectRemoved>) {
    metrics::record(LogCode::EFF002);
    let event = trigger.event();
    info!(
        code = ?LogCode::EFF002,
        event = "效果移除",
        instance = %event.instance_id,
        def_id = %event.def_id,
        target = %event.target_entity,
        reason = %event.reason,
        "效果移除"
    );
}

/// 效果 Tick 日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::EFF003, event = "效果Tick"))]
pub(crate) fn on_effect_ticked(trigger: On<EffectTicked>) {
    metrics::record(LogCode::EFF003);
    let event = trigger.event();
    debug!(
        code = ?LogCode::EFF003,
        event = "效果Tick",
        instance = %event.instance_id,
        target = %event.target_entity,
        tick = event.tick_number,
        total = ?event.total_ticks,
        "效果Tick"
    );
}

/// 效果免疫日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::EFF004, event = "效果免疫"))]
pub(crate) fn on_effect_immunity(trigger: On<EffectImmunityTriggered>) {
    metrics::record(LogCode::EFF004);
    let event = trigger.event();
    warn!(
        code = ?LogCode::EFF004,
        event = "效果免疫",
        def_id = %event.def_id,
        target = %event.target_entity,
        immune_tag = %event.immune_tag,
        "效果免疫"
    );
}
