// 统一移动执行系统：监听 MovementIntent，统一处理移动逻辑
// 实现意图与执行分离，确保 AI 和玩家使用相同的移动路径

use crate::character::{GridPosition, MovingUnit, spawn_path_arrows};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::GameplayTags;
use crate::map::{
    GameMap, OccupancyGrid, TerrainCostRegistry, TerrainGrid, TerrainRegistry, find_reachable_tiles,
    reconstruct_path,
};
use crate::turn::TurnPhase;
use crate::ui::events::{IntentSource, MovementIntent};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 移动执行系统 - 监听 MovementIntent，统一处理移动逻辑
pub fn movement_execution_system(
    mut commands: Commands,
    mut intent_reader: MessageReader<MovementIntent>,
    map: Res<GameMap>,
    terrain: (Res<TerrainGrid>, Res<TerrainRegistry>),
    occupancy: Res<OccupancyGrid>,
    cost_registry: Res<TerrainCostRegistry>,
    units: Query<(Entity, &Attributes, &GameplayTags, &GridPosition)>,
) {
    for intent in intent_reader.read() {
        execute_movement(
            &mut commands,
            intent,
            &map,
            &terrain.0,
            &terrain.1,
            &occupancy,
            &cost_registry,
            &units,
        );
    }
}

/// 统一的移动执行逻辑
fn execute_movement(
    commands: &mut Commands,
    intent: &MovementIntent,
    map: &GameMap,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    occupancy: &OccupancyGrid,
    cost_registry: &TerrainCostRegistry,
    units: &Query<(Entity, &Attributes, &GameplayTags, &GridPosition)>,
) {
    let Ok((_, attrs, tags, grid_pos)) = units.get(intent.entity) else {
        return;
    };

    let start_coord = grid_pos.coord;

    // 原地不动，跳过移动
    if start_coord == intent.target_coord {
        return;
    }

    let move_range = attrs.get(AttributeKind::MoveRange) as u32;
    let calculator = cost_registry.resolve_from_tags(tags);

    let reachable = find_reachable_tiles(
        start_coord,
        move_range,
        map,
        terrain_grid,
        terrain_registry,
        occupancy,
        Some(intent.entity),
        calculator,
    );

    // 验证目标在可达范围内
    if !reachable.contains_key(&intent.target_coord) {
        return;
    }

    let path = reconstruct_path(
        start_coord,
        intent.target_coord,
        &reachable,
        move_range,
        map,
        terrain_grid,
        terrain_registry,
        calculator,
    );

    // 验证路径有效性：非空且终点正确
    // 注意：reconstruct_path 返回的路径不包含起点，只包含从起点之后的格子
    if path.is_empty() || path.last() != Some(&intent.target_coord) {
        return;
    }

    spawn_path_arrows(commands, map, &path);

    // 使用固定的移动速度，每格 0.15 秒
    let move_speed = 0.15_f32;

    commands.entity(intent.entity).insert(MovingUnit {
        path,
        current_index: 0,
        speed: move_speed,
        elapsed: 0.0,
        next_phase: match intent.source {
            IntentSource::Player => TurnPhase::ActionMenu,
            IntentSource::Ai => TurnPhase::ExecuteAction,
        },
    });
}
