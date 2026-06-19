//! turn_logger — 回合事件日志 Observer
//!
//! 监听全局回合事件（`TurnStarted` / `TurnEnded`），生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::events::{TurnEnded, TurnStarted};
use crate::shared::diagnostics::LogCode;

/// 单位回合开始日志 Observer。
///
/// 监听 `TurnStarted` 事件，记录开始回合的单位。
pub(crate) fn on_turn_started(trigger: On<TurnStarted>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::BAT005,
        event = "turn_started",
        unit = ?event.unit,
        "turn_started"
    );
}

/// 单位回合结束日志 Observer。
///
/// 监听 `TurnEnded` 事件，记录结束回合的单位。
pub(crate) fn on_turn_ended(trigger: On<TurnEnded>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::BAT006,
        event = "turn_ended",
        unit = ?event.unit,
        "turn_ended"
    );
}
