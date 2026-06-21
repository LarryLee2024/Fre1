//! Cleanup — 渲染实体清理
//!
//! 地图卸载时清除所有渲染实体。

use bevy::prelude::*;

use super::overlay::OverlayRoot;
use super::spawn::MapRoot;

// ─── 清理辅助函数 ────────────────────────────────────────────────

/// 清除所有地图渲染实体。
///
/// 查找 MapRoot 标记 Entity 并 despawn 整个子树。
pub fn despawn_map_entities(mut commands: Commands, map_root_query: &Query<Entity, With<MapRoot>>) {
    for entity in map_root_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 清除所有覆盖层实体。
///
/// 查找 OverlayRoot 标记 Entity 并 despawn。
pub fn despawn_overlay_entities(
    mut commands: Commands,
    overlay_query: &Query<Entity, With<OverlayRoot>>,
) {
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn();
    }
}
