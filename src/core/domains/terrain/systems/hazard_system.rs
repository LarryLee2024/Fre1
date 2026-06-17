//! Hazard System — 陷阱触发系统
//!
//! 监听 TileEntered 事件，检查目标格子是否包含陷阱定义，
//! 验证触发条件，发出 HazardTriggered 事件。
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §5.3

use std::collections::HashSet;

use bevy::prelude::*;

use crate::core::domains::terrain::components::{HazardTriggeredState, TilePos, TileProperties};
use crate::core::domains::terrain::events::{HazardTriggered, TileEntered};
use crate::core::domains::terrain::resources::HazardZoneRegistry;

/// 响应 TileEntered 事件，检查陷阱触发条件。
///
/// 触发流程：
/// 1. 查找格子的 HazardZone 定义
/// 2. 检查触发条件（阵营排除、类型排除）
/// 3. 检查陷阱是否已被消耗（消耗型陷阱）
/// 4. 发出 HazardTriggered 事件
pub(crate) fn on_hazard_check(
    trigger: On<TileEntered>,
    tile_query: Query<(&TilePos, &TileProperties)>,
    hazard_state_query: Query<&HazardTriggeredState>,
    hazard_registry: Option<Res<HazardZoneRegistry>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let target_tile = event.tile;
    let entity = event.entity;

    // 没有注册的陷阱定义时跳过
    let Some(registry) = hazard_registry else {
        return;
    };

    // 查找目标格子的位置
    let tile_entity = tile_query
        .iter()
        .find(|(pos, _)| pos.is_same_tile(target_tile));

    let Some((_tile_pos, tile_props)) = tile_entity else {
        return;
    };

    // 检查格子上是否有可用的陷阱定义
    let matched_hazards: Vec<_> = registry
        .zones
        .iter()
        .filter(|zone| zone.matches_tile(tile_props))
        .collect();

    if matched_hazards.is_empty() {
        return;
    }

    // 检查实体是否已记录陷阱消耗状态
    let mut consumed_set: HashSet<String> = HashSet::new();
    if let Ok(state) = hazard_state_query.get(entity) {
        for hazard_id in &state.consumed_hazards {
            consumed_set.insert(hazard_id.clone());
        }
    }

    // 触发未消耗的陷阱
    for hazard in matched_hazards {
        if hazard.is_consumable && consumed_set.contains(&hazard.id) {
            // 消耗型陷阱已被触发，跳过
            continue;
        }

        info!(
            "[Terrain] Hazard triggered: tile=({},{}), hazard={}, entity={:?}",
            target_tile.x, target_tile.y, hazard.id, entity
        );

        commands.trigger(HazardTriggered {
            tile: target_tile,
            target: entity,
            hazard_id: hazard.id.clone(),
        });

        // 记录消耗型陷阱状态
        if hazard.is_consumable {
            // 如果实体还没有 HazardTriggeredState，为其添加
            if !consumed_set.contains(&hazard.id) {
                commands.entity(entity).insert(HazardTriggeredState {
                    consumed_hazards: {
                        let mut set = consumed_set.clone();
                        set.insert(hazard.id.clone());
                        set
                    },
                });
            }
        }
    }
}
