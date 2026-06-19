//! terrain_logger — Terrain 域日志 Observer
//!
//! 监听地形事件（进入、表面变化、陷阱、效果），生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::terrain::events::{
    HazardTriggered, SurfaceChanged, TerrainEffectApplied, TileEntered,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 单位进入格子日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER001, event = "tile_entered"))]
pub(crate) fn on_tile_entered(trigger: On<TileEntered>) {
    metrics::record(LogCode::TER001);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER001,
        event = "tile_entered",
        entity = ?event.entity,
        tile = ?event.tile,
        surface = ?event.surface,
        "tile_entered"
    );
}

/// 表面变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER002, event = "surface_changed"))]
pub(crate) fn on_surface_changed(trigger: On<SurfaceChanged>) {
    metrics::record(LogCode::TER002);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER002,
        event = "surface_changed",
        tile = ?event.tile,
        old = ?event.old_surface,
        new = ?event.new_surface,
        "surface_changed"
    );
}

/// 陷阱触发日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER003, event = "hazard_triggered"))]
pub(crate) fn on_hazard_triggered(trigger: On<HazardTriggered>) {
    metrics::record(LogCode::TER003);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER003,
        event = "hazard_triggered",
        tile = ?event.tile,
        target = ?event.target,
        hazard_id = %event.hazard_id,
        "hazard_triggered"
    );
}

/// 地形效果施加日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER004, event = "terrain_effect_applied"))]
pub(crate) fn on_terrain_effect_applied(trigger: On<TerrainEffectApplied>) {
    metrics::record(LogCode::TER004);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER004,
        event = "terrain_effect_applied",
        entity = ?event.entity,
        tile = ?event.tile,
        effect = %event.effect_id,
        "terrain_effect_applied"
    );
}
