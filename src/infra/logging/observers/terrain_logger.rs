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
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER001, event = "格子进入"))]
pub(crate) fn on_tile_entered(trigger: On<TileEntered>) {
    metrics::record(LogCode::TER001);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER001,
        event = "格子进入",
        entity = ?event.entity,
        tile = ?event.tile,
        surface = ?event.surface,
        "格子进入"
    );
}

/// 表面变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER002, event = "表面变化"))]
pub(crate) fn on_surface_changed(trigger: On<SurfaceChanged>) {
    metrics::record(LogCode::TER002);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER002,
        event = "表面变化",
        tile = ?event.tile,
        old = ?event.old_surface,
        new = ?event.new_surface,
        "表面变化"
    );
}

/// 陷阱触发日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER003, event = "陷阱触发"))]
pub(crate) fn on_hazard_triggered(trigger: On<HazardTriggered>) {
    metrics::record(LogCode::TER003);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER003,
        event = "陷阱触发",
        tile = ?event.tile,
        target = ?event.target,
        hazard_id = %event.hazard_id,
        "陷阱触发"
    );
}

/// 地形效果施加日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::TER004, event = "地形效果施加"))]
pub(crate) fn on_terrain_effect_applied(trigger: On<TerrainEffectApplied>) {
    metrics::record(LogCode::TER004);
    let event = trigger.event();
    info!(
        code = ?LogCode::TER004,
        event = "地形效果施加",
        entity = ?event.entity,
        tile = ?event.tile,
        effect = %event.effect_id,
        "地形效果施加"
    );
}
