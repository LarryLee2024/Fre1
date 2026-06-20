//! battle_logger — 战斗生命周期日志 Observer
//!
//! 监听全局战斗事件（`BattleStarted` / `BattleEnded`），生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::events::{BattleEnded, BattleStarted};
use crate::emit_info;
use crate::shared::diagnostics::LogCode;

/// 战斗开始日志 Observer。
#[tracing::instrument(skip_all, target = "domain.combat", fields(
    code = ?LogCode::BAT001,
    event = "battle_started",
))]
pub(crate) fn on_battle_started(_trigger: On<BattleStarted>) {
    emit_info!(LogCode::BAT001, "战斗开始");
}

/// 战斗结束日志 Observer。
#[tracing::instrument(skip_all, target = "domain.combat", fields(
    code = ?LogCode::BAT002,
    event = "battle_ended",
))]
pub(crate) fn on_battle_ended(trigger: On<BattleEnded>) {
    let event = trigger.event();
    emit_info!(
        LogCode::BAT002,
        victory = event.victory,
        "战斗结束",
    );
}
