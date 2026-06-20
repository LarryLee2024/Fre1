//! ability_logger — 技能事件日志 Observer
//!
//! 监听技能生命周期事件（激活/完成/取消/冷却），生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::capabilities::ability::events::{
    AbilityActivated, AbilityCancelled, AbilityCompleted, AbilityCooldownStarted,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 技能激活日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ABL001, event = "技能激活"), target = "ability")]
pub(crate) fn on_ability_activated(trigger: On<AbilityActivated>) {
    metrics::record(LogCode::ABL001);
    let event = trigger.event();
    info!(
        code = ?LogCode::ABL001,
        event = "技能激活",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        context = %event.context_desc,
        "技能激活"
    );
}

/// 技能完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ABL002, event = "技能完成"), target = "ability")]
pub(crate) fn on_ability_completed(trigger: On<AbilityCompleted>) {
    metrics::record(LogCode::ABL002);
    let event = trigger.event();
    info!(
        code = ?LogCode::ABL002,
        event = "技能完成",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        result = %event.result,
        "技能完成"
    );
}

/// 技能取消日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ABL003, event = "技能取消"), target = "ability")]
pub(crate) fn on_ability_cancelled(trigger: On<AbilityCancelled>) {
    metrics::record(LogCode::ABL003);
    let event = trigger.event();
    info!(
        code = ?LogCode::ABL003,
        event = "技能取消",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        reason = %event.reason,
        "技能取消"
    );
}

/// 技能冷却开始日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ABL004, event = "技能冷却开始"), target = "ability")]
pub(crate) fn on_ability_cooldown_started(trigger: On<AbilityCooldownStarted>) {
    metrics::record(LogCode::ABL004);
    let event = trigger.event();
    info!(
        code = ?LogCode::ABL004,
        event = "技能冷却开始",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        duration = event.cooldown_duration,
        shared = ?event.shared_group,
        "技能冷却开始"
    );
}
