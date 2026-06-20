//! terrain_logger — Terrain 域日志 Observer
//!
//! 监听地形事件（进入、表面变化、陷阱、效果），生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::terrain::events::{
    HazardTriggered, SurfaceChanged, TerrainEffectApplied, TileEntered,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 单位进入格子日志 Observer。
#[tracing::instrument(skip_all, target = "domain.terrain", fields(
    code = ?LogCode::TER001,
    event = "entity_entered_tile",
))]
pub(crate) fn on_tile_entered(trigger: On<TileEntered>) {
    metrics::record(LogCode::TER001);
    let event = trigger.event();
    info!(
        target = "domain.terrain",
        entity = ?event.entity,
        tile = ?event.tile,
        surface = ?event.surface,
        "格子进入",
    );
}

/// 表面变化日志 Observer。
#[tracing::instrument(skip_all, target = "domain.terrain", fields(
    code = ?LogCode::TER002,
    event = "tile_surface_changed",
))]
pub(crate) fn on_surface_changed(trigger: On<SurfaceChanged>) {
    metrics::record(LogCode::TER002);
    let event = trigger.event();
    info!(
        target = "domain.terrain",
        tile = ?event.tile,
        old = ?event.old_surface,
        new = ?event.new_surface,
        "表面变化",
    );
}

/// 陷阱触发日志 Observer。
#[tracing::instrument(skip_all, target = "domain.terrain", fields(
    code = ?LogCode::TER003,
    event = "trap_triggered",
))]
pub(crate) fn on_hazard_triggered(trigger: On<HazardTriggered>) {
    metrics::record(LogCode::TER003);
    let event = trigger.event();
    info!(
        target = "domain.terrain",
        tile = ?event.tile,
        target = ?event.target,
        hazard_id = %event.hazard_id,
        "陷阱触发",
    );
}

/// 地形效果施加日志 Observer。
#[tracing::instrument(skip_all, target = "domain.terrain", fields(
    code = ?LogCode::TER004,
    event = "terrain_effect_applied",
))]
pub(crate) fn on_terrain_effect_applied(trigger: On<TerrainEffectApplied>) {
    metrics::record(LogCode::TER004);
    let event = trigger.event();
    info!(
        target = "domain.terrain",
        entity = ?event.entity,
        tile = ?event.tile,
        effect = %event.effect_id,
        "地形效果施加",
    );
}
