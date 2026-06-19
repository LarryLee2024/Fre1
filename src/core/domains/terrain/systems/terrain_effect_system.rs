//! Terrain Effect System — 地形效果应用系统
//!
//! 监听 TileEntered 事件，根据进入格子的表面类型施加对应的地形效果。
//! 地形效果通过 TerrainEffectApplied 事件发出，由 Effect 领域处理实际效果生命周期。
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §5.1

use bevy::prelude::*;

use crate::core::domains::terrain::components::{SurfaceType, TilePos, TileProperties};
use crate::core::domains::terrain::events::{TerrainEffectApplied, TileEntered};
use crate::core::domains::terrain::resources::TileEntityMap;
use crate::shared::ids::DefinitionId;

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
    tile_props_query: Query<&TileProperties>,
    tile_map: Res<TileEntityMap>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let target_tile = event.tile;
    let entity = event.entity;

    // 通过 TileEntityMap 空间索引 O(1) 查找格子
    let surface_effect = tile_map
        .get(&target_tile)
        .and_then(|tile_entity| tile_props_query.get(tile_entity).ok())
        .and_then(|props| surface_to_effect_id(props.surface));

    // 如果表面有对应的地形效果，发出 TerrainEffectApplied 事件
    if let Some(effect_id) = surface_effect {
        commands.trigger(TerrainEffectApplied {
            entity,
            tile: target_tile,
            effect_id,
        });
    }
}

/// 通知单位进入格子，触发 TileEntered 事件。
///
/// 由移动系统或传送逻辑在单位进入新格子时调用。
pub(crate) fn notify_tile_entered(
    mut commands: Commands,
    entity: Entity,
    tile: TilePos,
    surface: SurfaceType,
) {
    commands.trigger(TileEntered {
        entity,
        tile,
        surface,
    });
}

/// 表面类型到 EffectDefId 的映射常量。
///
/// TODO[P2][Terrain]: 待 Registry 内容系统定型后从配置加载
///   当前为硬编码占位 ID，需与 Effect 领域对齐 ID 分配方案。
const EFFECT_POISON: &str = "eff_000001";
const EFFECT_BURNING: &str = "eff_000002";
const EFFECT_LAVA: &str = "eff_000003";

/// 表面类型到 EffectDefId 的映射。
///
/// 非所有表面都有地形效果（如 Normal、Ice 等仅影响移动）。
/// 返回 None 表示无需施加效果。
fn surface_to_effect_id(surface: SurfaceType) -> Option<DefinitionId> {
    match surface {
        SurfaceType::Poison => Some(DefinitionId::new(EFFECT_POISON)),
        SurfaceType::Burning => Some(DefinitionId::new(EFFECT_BURNING)),
        SurfaceType::Lava => Some(DefinitionId::new(EFFECT_LAVA)),
        _ => None,
    }
}
