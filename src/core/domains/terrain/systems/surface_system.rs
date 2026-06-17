//! Surface System — 表面变化处理与恢复系统
//!
//! 1. 监听 SurfaceChanged 事件，更新 TileProperties 的表面状态
//! 2. 定时恢复系统：递减 SurfaceOverride 的剩余回合数，到期自动恢复
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §5.2

use bevy::prelude::*;

use crate::core::domains::terrain::components::{SurfaceOverride, TilePos, TileProperties};
use crate::core::domains::terrain::events::SurfaceChanged;

/// 响应 SurfaceChanged 事件，更新目标格子的 TileProperties.surface。
pub(crate) fn on_surface_changed(
    trigger: On<SurfaceChanged>,
    mut tile_query: Query<(&TilePos, &mut TileProperties)>,
) {
    let event = trigger.event();

    for (tile_pos, mut props) in tile_query.iter_mut() {
        if tile_pos.is_same_tile(event.tile) {
            props.surface = event.new_surface;
            info!(
                "[Terrain] SurfaceChanged: tile=({},{}), {:?} → {:?}",
                tile_pos.x, tile_pos.y, event.old_surface, event.new_surface
            );
            return;
        }
    }
}

/// 递减 SurfaceOverride 的剩余回合数，到期时恢复原始表面。
///
/// TODO[P2][Terrain]: 已从 Update 调度移除，待 D-9 Turn 系统实现后通过 OnTurnEnd 驱动
///   当前仅当显式调用时执行回合递减。在 D-9 完成前，表面覆盖不会自动到期。
///   届时在 TerrainPlugin 中注册：app.add_systems(Update, surface_recovery_system)
pub(crate) fn surface_recovery_system(
    mut commands: Commands,
    mut surface_query: Query<(
        Entity,
        &mut SurfaceOverride,
        &mut TileProperties,
        Option<&TilePos>,
    )>,
) {
    for (entity, mut override_, mut props, tile_pos) in surface_query.iter_mut() {
        // 定时恢复：递减剩余回合
        if let Some(ref mut remaining) = override_.remaining_duration {
            if *remaining > 0 {
                *remaining -= 1;
            }
        }

        // 检查是否到期（剩余回合为 0 或已过期）
        if override_.is_expired() {
            // 恢复原始表面
            props.surface = override_.original;
            let pos = tile_pos.copied().unwrap_or(TilePos::new(0, 0));
            info!(
                "[Terrain] Surface recovered: tile=({},{}), restored to {:?}",
                pos.x, pos.y, override_.original
            );

            // 发射 SurfaceChanged 事件通知恢复
            commands.trigger(SurfaceChanged {
                tile: pos,
                old_surface: override_.current,
                new_surface: override_.original,
            });

            // 移除 SurfaceOverride 组件
            commands.entity(entity).remove::<SurfaceOverride>();
        }
    }
}
