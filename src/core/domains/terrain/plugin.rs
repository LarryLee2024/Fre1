//! TerrainPlugin — 地形领域 Plugin
//!
//! 注册地形组件、事件和系统。
//! 处理地形属性、表面变化、陷阱触发、地形效果。
//!
//! 详见 ADR-022

use bevy::prelude::*;

use super::components::{
    HazardTriggeredState, SurfaceOverride, TerrainAttachEffect, TilePos, TileProperties,
};
use super::resources::{HazardZoneRegistry, TileEntityMap};
use super::systems::hazard_system::on_hazard_check;
use super::systems::on_turn_end_surface_recovery;
use super::systems::surface_system::on_surface_changed;
use super::systems::terrain_effect_system::on_tile_entered;
use crate::register_domain_types;
use crate::shared::game_state::GameState;

/// 地形领域 Plugin——注册地形组件、表面变化和陷阱触发系统。
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(
            app,
            [
                TilePos,
                TileProperties,
                SurfaceOverride,
                TerrainAttachEffect,
                HazardTriggeredState,
            ]
        );

        // ── 初始化 Resource ──
        app.init_resource::<HazardZoneRegistry>();
        app.init_resource::<TileEntityMap>();

        // ── 注册 Observer System ──
        // TileEntered → 地形效果应用
        app.add_observer(on_tile_entered);
        // TileEntered → 陷阱检测
        app.add_observer(on_hazard_check);
        // SurfaceChanged → 格子属性更新
        app.add_observer(on_surface_changed);

        // ── 注册常规 System ──
        // 空间索引维护：在 PostUpdate 中重建 TilePos → Entity 映射
        app.add_systems(
            PostUpdate,
            TileEntityMap::update.run_if(in_state(GameState::TacticalMap)),
        );

        // ── 注册 Observer ──
        // OnturnEnd → 表面覆盖回合计数递减
        app.add_observer(on_turn_end_surface_recovery);
    }
}
