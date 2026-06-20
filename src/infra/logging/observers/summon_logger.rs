//! summon_logger — Summon 域日志 Observer
//!
//! 监听召唤物创建、消失、指令事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::summon::events::{
    SummonCommand, SummonCreated, SummonExpired, SummonSlotChanged,
};
use crate::emit_info;
use crate::shared::diagnostics::LogCode;

/// 召唤物创建日志 Observer。
#[tracing::instrument(skip_all, target = "domain.summon", fields(
    code = ?LogCode::SUM001,
    event = "summon_created",
))]
pub(crate) fn on_summon_created(trigger: On<SummonCreated>) {
    let event = trigger.event();
    emit_info!(
        LogCode::SUM001,
        caster = ?event.caster,
        summon = ?event.summon_entity,
        template = %event.template_id,
        "召唤物创建",
    );
}

/// 召唤物消失日志 Observer。
#[tracing::instrument(skip_all, target = "domain.summon", fields(
    code = ?LogCode::SUM002,
    event = "summon_vanished",
))]
pub(crate) fn on_summon_expired(trigger: On<SummonExpired>) {
    let event = trigger.event();
    emit_info!(
        LogCode::SUM002,
        caster = ?event.caster,
        summon = ?event.summon_entity,
        reason = ?event.reason,
        "召唤物消失",
    );
}

/// 召唤指令日志 Observer。
#[tracing::instrument(skip_all, target = "domain.summon", fields(
    code = ?LogCode::SUM003,
    event = "summon_commanded",
))]
pub(crate) fn on_summon_command(trigger: On<SummonCommand>) {
    let event = trigger.event();
    emit_info!(
        LogCode::SUM003,
        caster = ?event.caster,
        summon = ?event.summon_entity,
        command = ?event.command_type,
        "召唤指令",
    );
}

/// 召唤槽位变化日志 Observer。
#[tracing::instrument(skip_all, target = "domain.summon", fields(
    code = ?LogCode::SUM004,
    event = "summon_slot_changed",
))]
pub(crate) fn on_summon_slot_changed(trigger: On<SummonSlotChanged>) {
    let event = trigger.event();
    emit_info!(
        LogCode::SUM004,
        caster = ?event.caster,
        used = event.slots_used,
        max = event.slots_max,
        "召唤槽位变化",
    );
}
