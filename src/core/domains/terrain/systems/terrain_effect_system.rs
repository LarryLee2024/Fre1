//! Terrain Effect System — 地形效果应用系统
//!
//! 监听 TileEntered 事件，根据进入格子的表面类型施加对应的地形效果。
//! 地形效果通过 TerrainEffectApplied 事件发出，由 Effect 领域处理实际效果生命周期。
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §5.1

use bevy::prelude::*;

use crate::core::domains::terrain::components::{SurfaceType, TilePos, TileProperties};
use crate::core::domains::terrain::events::{TerrainEffectApplied, TileEntered};

/// 响应 TileEntered 事件，检查格子表面并施加地形效果。
///
/// 触发方式：commands.trigger(TileEntered { entity, tile, surface })
///
/// 表面类型 → 地形效果映射：
/// - Poison → eff_000001（中毒效果）
/// - Burning → eff_000002（灼烧效果）
/// - Lava → eff_000003（岩浆伤害）
/// - Oil → 无直接效果（可被点燃）
pub(crate) fn on_tile_entered(
    trigger: On<TileEntered>,
    tile_query: Query<(&TilePos, &TileProperties)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let target_tile = event.tile;
    let entity = event.entity;

    // 查找格子实体，检查表面类型
    let surface_effect = tile_query
        .iter()
        .find(|(pos, _)| pos.is_same_tile(target_tile))
        .and_then(|(_, props)| surface_to_effect_id(props.surface));

    // 如果表面有对应的地形效果，发出 TerrainEffectApplied 事件
    if let Some(effect_id) = surface_effect {
        info!(
            "[Terrain] TileEntered: entity={:?}, tile=({},{}), surface={:?} → effect={}",
            entity, target_tile.x, target_tile.y, event.surface, effect_id
        );
        commands.trigger(TerrainEffectApplied {
            entity,
            tile: target_tile,
            effect_id,
        });
    }
}

/// 表面类型到 EffectDefId 的映射。
///
/// 非所有表面都有地形效果（如 Normal、Ice 等仅影响移动）。
/// 返回 None 表示无需施加效果。
fn surface_to_effect_id(surface: SurfaceType) -> Option<String> {
    match surface {
        SurfaceType::Poison => Some("eff_000001".to_string()),
        SurfaceType::Burning => Some("eff_000002".to_string()),
        SurfaceType::Lava => Some("eff_000003".to_string()),
        _ => None,
    }
}
