//! Hazard System — 陷阱触发系统
//!
//! 监听 TileEntered 事件，检查目标格子是否包含陷阱定义，
//! 验证触发条件，发出 HazardTriggered 事件。
//!
//! 详见 docs/02-domain/domains/terrain_domain.md §5.3

use std::collections::HashSet;

use bevy::prelude::*;

use crate::core::domains::terrain::components::{HazardTriggeredState, TileProperties};
use crate::core::domains::terrain::events::{HazardTriggered, TileEntered};
use crate::core::domains::terrain::resources::{HazardZoneRegistry, TileEntityMap};

/// 响应 TileEntered 事件，检查陷阱触发条件。
///
/// 触发流程：
/// 1. 查找格子的 HazardZone 定义
/// 2. 检查触发条件（阵营排除、类型排除）
/// 3. 检查陷阱是否已被消耗（消耗型陷阱）
/// 4. 发出 HazardTriggered 事件
pub(crate) fn on_hazard_check(
    trigger: On<TileEntered>,
    tile_props_query: Query<&TileProperties>,
    hazard_state_query: Query<&HazardTriggeredState>,
    hazard_registry: Option<Res<HazardZoneRegistry>>,
    tile_map: Res<TileEntityMap>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let target_tile = event.tile;
    let entity = event.entity;

    // 没有注册的陷阱定义时跳过
    let Some(registry) = hazard_registry else {
        return;
    };

    // 通过 TileEntityMap 空间索引 O(1) 查找格子属性
    let Some(tile_props) = tile_map
        .get(&target_tile)
        .and_then(|tile_entity| tile_props_query.get(tile_entity).ok())
    else {
        return;
    };

    // HazardRegistry 按 zone 定义匹配当前格子的地形属性
    let matched_hazards: Vec<_> = registry
        .zones
        .iter()
        .filter(|zone| zone.matches_tile(tile_props))
        .collect();

    if matched_hazards.is_empty() {
        return;
    }

    // HazardTriggeredState 记录已消耗的陷阱，防止同一格子的消耗型陷阱重复触发
    let mut consumed_set: HashSet<String> = HashSet::new();
    if let Ok(state) = hazard_state_query.get(entity) {
        for hazard_id in &state.consumed_hazards {
            consumed_set.insert(hazard_id.clone());
        }
    }

    // 消耗型陷阱触发后需更新状态，确保同一实体不会重复触发同一陷阱
    for hazard in matched_hazards {
        if hazard.is_consumable && consumed_set.contains(&hazard.id) {
            // 消耗型陷阱已被触发，跳过
            continue;
        }

        commands.trigger(HazardTriggered {
            tile: target_tile,
            target: entity,
            hazard_id: hazard.id.clone(),
        });

        // 消耗型陷阱首次触发时插入状态组件，后续经过时走 consumed_set 跳过
        if hazard.is_consumable {
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
