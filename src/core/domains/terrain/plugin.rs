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
use super::systems::surface_system::on_surface_changed;
use super::systems::terrain_effect_system::on_tile_entered;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        app.register_type::<TilePos>();
        app.register_type::<TileProperties>();
        app.register_type::<SurfaceOverride>();
        app.register_type::<TerrainAttachEffect>();
        app.register_type::<HazardTriggeredState>();

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
        app.add_systems(PostUpdate, TileEntityMap::update);

        // 表面恢复：已从 Update 调度移除
        // TODO[P2][Terrain]: 待 D-9 Turn 系统实现后接入 OnTurnEnd 事件
        //   surface_recovery_system 函数已实现（src/.../surface_system.rs），
        //   当前不注册以防每帧耗尽持续回合。D-9 完成后在此处注册即可。
    }
}
