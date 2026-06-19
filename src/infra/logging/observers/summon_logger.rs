//! summon_logger — Summon 域日志 Observer
//!
//! 监听召唤物创建、消失、指令事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::summon::events::{
    SummonCommand, SummonCreated, SummonExpired, SummonSlotChanged,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 召唤物创建日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::SUM001, event = "summon_created"))]
pub(crate) fn on_summon_created(trigger: On<SummonCreated>) {
    metrics::record(LogCode::SUM001);
    let event = trigger.event();
    info!(
        code = ?LogCode::SUM001,
        event = "summon_created",
        caster = ?event.caster,
        summon = ?event.summon_entity,
        template = %event.template_id,
        "summon_created"
    );
}

/// 召唤物消失日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::SUM002, event = "summon_expired"))]
pub(crate) fn on_summon_expired(trigger: On<SummonExpired>) {
    metrics::record(LogCode::SUM002);
    let event = trigger.event();
    info!(
        code = ?LogCode::SUM002,
        event = "summon_expired",
        caster = ?event.caster,
        summon = ?event.summon_entity,
        reason = ?event.reason,
        "summon_expired"
    );
}

/// 召唤指令日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::SUM003, event = "summon_command"))]
pub(crate) fn on_summon_command(trigger: On<SummonCommand>) {
    metrics::record(LogCode::SUM003);
    let event = trigger.event();
    info!(
        code = ?LogCode::SUM003,
        event = "summon_command",
        caster = ?event.caster,
        summon = ?event.summon_entity,
        command = ?event.command_type,
        "summon_command"
    );
}

/// 召唤槽位变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::SUM004, event = "summon_slot_changed"))]
pub(crate) fn on_summon_slot_changed(trigger: On<SummonSlotChanged>) {
    metrics::record(LogCode::SUM004);
    let event = trigger.event();
    info!(
        code = ?LogCode::SUM004,
        event = "summon_slot_changed",
        caster = ?event.caster,
        used = event.slots_used,
        max = event.slots_max,
        "summon_slot_changed"
    );
}
