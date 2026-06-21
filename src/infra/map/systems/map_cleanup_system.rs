//! MapCleanupSystem — 地图卸载清理
//!
//! 地图卸载时清除 GridMap Resource 和渲染实体。
//! V1 使用直接函数调用，后续接入 Bevy 0.19 Observer 模式。

use bevy::prelude::*;

use crate::core::domains::tactical::resources::GridMap;

use super::super::renderer::overlay::OverlayRoot;
use super::super::renderer::spawn::MapRoot;

/// 清除地图运行时状态（Resource + 渲染实体）。
///
/// 在场景切换或地图卸载时手动调用。
/// 后续版本将接入 MapUnloadedEvent Observer。
pub fn cleanup_map_world(mut commands: Commands, map_root_query: &Query<Entity, With<MapRoot>>) {
    // 1. 移除 GridMap Resource
    commands.remove_resource::<GridMap>();

    // 2. Despawn MapRoot 及所有 Tile Entities
    for entity in map_root_query.iter() {
        commands.entity(entity).despawn();
    }

    info!(target: "map", "[Map] 地图资源清理完成");
}

/// 清除覆盖层实体。
pub fn cleanup_overlay(mut commands: Commands, overlay_query: &Query<Entity, With<OverlayRoot>>) {
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn();
    }
}
