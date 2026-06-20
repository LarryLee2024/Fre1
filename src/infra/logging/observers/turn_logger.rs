//! turn_logger — 回合事件日志 Observer
//!
//! 监听全局回合事件（`TurnStarted` / `TurnEnded`），生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::events::{TurnEnded, TurnStarted};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 单位回合开始日志 Observer。
///
/// 监听 `TurnStarted` 事件，记录开始回合的单位。
#[tracing::instrument(skip_all, fields(code = ?LogCode::BAT005, event = "回合开始"))]
pub(crate) fn on_turn_started(trigger: On<TurnStarted>) {
    metrics::record(LogCode::BAT005);
    let event = trigger.event();
    info!(
        code = ?LogCode::BAT005,
        event = "回合开始",
        unit = ?event.unit,
        "回合开始"
    );
}

/// 单位回合结束日志 Observer。
///
/// 监听 `TurnEnded` 事件，记录结束回合的单位。
#[tracing::instrument(skip_all, fields(code = ?LogCode::BAT006, event = "回合结束"))]
pub(crate) fn on_turn_ended(trigger: On<TurnEnded>) {
    metrics::record(LogCode::BAT006);
    let event = trigger.event();
    info!(
        code = ?LogCode::BAT006,
        event = "回合结束",
        unit = ?event.unit,
        "回合结束"
    );
}
