//! battle_logger — 战斗生命周期日志 Observer
//!
//! 监听全局战斗事件（`BattleStarted` / `BattleEnded`），生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::events::{BattleEnded, BattleStarted};
use crate::shared::diagnostics::LogCode;

/// 战斗开始日志 Observer。
///
/// 监听 `BattleStarted` 事件，输出结构化 INFO 日志。
pub(crate) fn on_battle_started(_trigger: On<BattleStarted>) {
    info!(
        code = ?LogCode::BAT001,
        event = "battle_started",
        "battle_started"
    );
}

/// 战斗结束日志 Observer。
///
/// 监听 `BattleEnded` 事件，记录战斗结果（胜利/失败）。
pub(crate) fn on_battle_ended(trigger: On<BattleEnded>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::BAT002,
        event = "battle_ended",
        victory = event.victory,
        "battle_ended"
    );
}
