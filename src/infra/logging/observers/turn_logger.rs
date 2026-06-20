//! turn_logger — 回合事件日志 Observer
//!
//! 监听全局回合事件（`TurnStarted` / `TurnEnded`），生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::events::{TurnEnded, TurnStarted};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 单位回合开始日志 Observer。
#[tracing::instrument(skip_all, target = "domain.combat", fields(
    code = ?LogCode::BAT005,
    event = "unit_turn_started",
))]
pub(crate) fn on_turn_started(trigger: On<TurnStarted>) {
    metrics::record(LogCode::BAT005);
    let event = trigger.event();
    info!(
        target = "domain.combat",
        unit = ?event.unit,
        "回合开始",
    );
}

/// 单位回合结束日志 Observer。
#[tracing::instrument(skip_all, target = "domain.combat", fields(
    code = ?LogCode::BAT006,
    event = "unit_turn_ended",
))]
pub(crate) fn on_turn_ended(trigger: On<TurnEnded>) {
    metrics::record(LogCode::BAT006);
    let event = trigger.event();
    info!(
        target = "domain.combat",
        unit = ?event.unit,
        "回合结束",
    );
}
