//! ability_logger — 技能事件日志 Observer
//!
//! 监听技能生命周期事件（激活/完成/取消/冷却），生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量
//! - 不使用 `context_desc` 等高基数字段

use bevy::prelude::*;

use crate::core::capabilities::ability::events::{
    AbilityActivated, AbilityCancelled, AbilityCompleted, AbilityCooldownStarted,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 技能激活日志 Observer。
///
/// 注意：`context_desc` 属于高基数字段，已被移除。使用 `spec_id` 替代。
#[tracing::instrument(skip_all, target = "domain.ability", fields(
    code = ?LogCode::ABL001,
    event = "ability_activated",
))]
pub(crate) fn on_ability_activated(trigger: On<AbilityActivated>) {
    metrics::record(LogCode::ABL001);
    let event = trigger.event();
    info!(
        target = "domain.ability",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        "技能激活",
    );
}

/// 技能完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.ability", fields(
    code = ?LogCode::ABL002,
    event = "ability_completed",
))]
pub(crate) fn on_ability_completed(trigger: On<AbilityCompleted>) {
    metrics::record(LogCode::ABL002);
    let event = trigger.event();
    info!(
        target = "domain.ability",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        result = %event.result,
        "技能完成",
    );
}

/// 技能取消日志 Observer。
#[tracing::instrument(skip_all, target = "domain.ability", fields(
    code = ?LogCode::ABL003,
    event = "ability_cancelled",
))]
pub(crate) fn on_ability_cancelled(trigger: On<AbilityCancelled>) {
    metrics::record(LogCode::ABL003);
    let event = trigger.event();
    info!(
        target = "domain.ability",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        reason = %event.reason,
        "技能取消",
    );
}

/// 技能冷却开始日志 Observer。
#[tracing::instrument(skip_all, target = "domain.ability", fields(
    code = ?LogCode::ABL004,
    event = "cooldown_started",
))]
pub(crate) fn on_ability_cooldown_started(trigger: On<AbilityCooldownStarted>) {
    metrics::record(LogCode::ABL004);
    let event = trigger.event();
    info!(
        target = "domain.ability",
        entity = ?event.entity,
        spec_id = %event.spec_id,
        duration = event.cooldown_duration,
        shared = ?event.shared_group,
        "技能冷却开始",
    );
}
