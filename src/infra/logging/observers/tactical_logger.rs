//! tactical_logger — Tactical 域日志 Observer
//!
//! 监听单位移动、位置变更事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::tactical::events::{PositionChanged, UnitMoved};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 单位移动完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.tactical", fields(
    code = ?LogCode::TAC001,
    event = "movement_completed",
))]
pub(crate) fn on_unit_moved(trigger: On<UnitMoved>) {
    metrics::record(LogCode::TAC001);
    let event = trigger.event();
    info!(
        target = "domain.tactical",
        entity = ?event.entity,
        from = ?event.from,
        to = ?event.to,
        remaining_mp = event.remaining_mp,
        "单位移动",
    );
}

/// 单位位置变更日志 Observer。
#[tracing::instrument(skip_all, target = "domain.tactical", fields(
    code = ?LogCode::TAC005,
    event = "position_changed",
))]
pub(crate) fn on_position_changed(trigger: On<PositionChanged>) {
    metrics::record(LogCode::TAC005);
    let event = trigger.event();
    info!(
        target = "domain.tactical",
        entity = ?event.entity,
        new_pos = ?event.new_pos,
        "位置变更",
    );
}
