// 统一移动执行系统：监听 MovementIntent，统一处理移动逻辑
// 实现意图与执行分离，确保 AI 和玩家使用相同的移动路径

use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::character::{GridPosition, MovingUnit, spawn_path_arrows};
use crate::core::map::{
    GameMap, OccupancyGrid, TerrainCostRegistry, TerrainGrid, TerrainRegistry,
    find_reachable_tiles, reconstruct_path,
};
use crate::core::movement::events::{IntentSource, MovementIntent};
use crate::core::tag::GameplayTags;
use crate::core::turn::TurnPhase;
use crate::infrastructure::logging::events::UnitMoved;
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
    mut log_writer: MessageWriter<UnitMoved>,
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
            &mut log_writer,
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
    log_writer: &mut MessageWriter<UnitMoved>,
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

    // 使用 find_reachable_tiles 计算可达范围，但不依赖 OccupancyGrid 验证目标
    // （OccupancyGrid 可能在 show_move_range 和此系统之间被更新）
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

    // 直接尝试寻路，如果可达范围包含目标则使用，否则直接寻路
    let path = if reachable.contains_key(&intent.target_coord) {
        reconstruct_path(
            start_coord,
            intent.target_coord,
            &reachable,
            move_range,
            map,
            terrain_grid,
            terrain_registry,
            calculator,
        )
    } else {
        // OccupancyGrid 可能已更新，直接用曼哈顿距离+地形成本寻路
        // 不依赖 OccupancyGrid 的可达范围计算
        find_path_ignore_occupancy(
            start_coord,
            intent.target_coord,
            move_range,
            map,
            terrain_grid,
            terrain_registry,
            calculator,
        )
    };

    // 验证路径有效性
    if path.is_empty() || path.last() != Some(&intent.target_coord) {
        return;
    }

    spawn_path_arrows(commands, map, &path);

    let move_speed = 0.15_f32;
    let path_len = path.len();

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

    log_writer.write(UnitMoved {
        entity: intent.entity,
        unit_name: String::new(), // TODO: 查询 UnitName 组件
        from: start_coord,
        to: intent.target_coord,
    });
}

/// 简单寻路：不考虑单位占用，只考虑地形和移动范围
/// 用于 OccupancyGrid 已被更新但 show_move_range 计算时还未更新的情况
fn find_path_ignore_occupancy(
    start: IVec2,
    target: IVec2,
    move_range: u32,
    map: &GameMap,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    calculator: &dyn crate::core::map::TerrainCostCalculator,
) -> Vec<IVec2> {
    use std::collections::{HashMap, VecDeque};

    let directions = [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ];

    let mut came_from: HashMap<IVec2, IVec2> = HashMap::new();
    let mut cost_so_far: HashMap<IVec2, u32> = HashMap::new();
    let mut queue = VecDeque::new();

    cost_so_far.insert(start, 0);
    queue.push_back((start, 0u32));

    while let Some((pos, current_cost)) = queue.pop_front() {
        if pos == target {
            break;
        }

        for dir in &directions {
            let next = pos + *dir;
            if !map.is_in_bounds(next) {
                continue;
            }

            let terrain_id = terrain_grid.get(next).unwrap_or("plain");
            let base_cost = terrain_registry.get(terrain_id).and_then(|d| d.move_cost);
            let cost = match calculator.cost(terrain_id, base_cost) {
                Some(c) => c,
                None => continue,
            };

            let new_cost = current_cost + cost;
            if new_cost > move_range {
                continue;
            }
            if !cost_so_far.contains_key(&next)
                || cost_so_far.get(&next).is_some_and(|&old| new_cost < old)
            {
                cost_so_far.insert(next, new_cost);
                came_from.insert(next, pos);
                queue.push_back((next, new_cost));
            }
        }
    }

    // 重建路径
    let mut path = Vec::new();
    let mut current = target;
    while current != start {
        path.push(current);
        match came_from.get(&current) {
            Some(&prev) => current = prev,
            None => return Vec::new(),
        }
    }
    path.reverse();
    path
}
