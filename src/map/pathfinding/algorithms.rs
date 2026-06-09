// 寻路核心算法：BFS 计算可移动范围与路径回溯

use super::cost::TerrainCostCalculator;
use crate::map::data::TerrainRegistry;
use crate::map::grid::GameMap;
use crate::map::runtime::{OccupancyGrid, TerrainGrid};
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

/// 从 BFS 可达结果回溯最短路径
/// 返回从 start（不含）到 target（含）的坐标序列
pub fn reconstruct_path(
    start: IVec2,
    target: IVec2,
    reachable: &HashMap<IVec2, u32>,
    move_points: u32,
    map: &GameMap,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    calculator: &dyn TerrainCostCalculator,
) -> Vec<IVec2> {
    if start == target || !reachable.contains_key(&target) {
        return vec![target];
    }

    let directions = [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ];

    let mut path = vec![target];
    let mut current = target;
    let mut remaining = reachable[&target];

    while current != start {
        let mut best_prev = None;
        let mut best_remaining = remaining;

        for dir in &directions {
            let prev = current - *dir;
            if prev == start {
                best_prev = Some(prev);
                break;
            }
            if let Some(&prev_remaining) = reachable.get(&prev) {
                let terrain_id = terrain_grid.get(current).unwrap_or("plain");
                let base_cost = terrain_registry
                    .get(terrain_id)
                    .and_then(|def| def.move_cost);
                let cost = match calculator.cost(terrain_id, base_cost) {
                    Some(c) => c,
                    None => continue,
                };
                if prev_remaining == remaining + cost && prev_remaining > best_remaining {
                    best_prev = Some(prev);
                    best_remaining = prev_remaining;
                }
            }
        }

        match best_prev {
            Some(prev) => {
                current = prev;
                remaining = best_remaining;
                path.push(current);
            }
            None => break,
        }
    }

    path.reverse();
    if path.first() == Some(&start) {
        path.remove(0);
    }
    path
}

/// 寻路结果：可到达的格子及其剩余移动力
/// 直接从 TerrainGrid + OccupancyGrid 读取数据
pub fn find_reachable_tiles(
    start: IVec2,
    move_points: u32,
    map: &GameMap,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    occupancy: &OccupancyGrid,
    moving_entity: Option<Entity>,
    calculator: &dyn TerrainCostCalculator,
) -> HashMap<IVec2, u32> {
    let mut reachable = HashMap::new();
    let mut queue = VecDeque::new();

    reachable.insert(start, move_points);
    queue.push_back((start, move_points));

    let directions = [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ];

    while let Some((pos, remaining)) = queue.pop_front() {
        for dir in &directions {
            let next = pos + *dir;

            if !map.is_in_bounds(next) {
                continue;
            }

            let terrain_id = terrain_grid.get(next).unwrap_or("plain");
            let base_cost = terrain_registry
                .get(terrain_id)
                .and_then(|def| def.move_cost);

            let cost = match calculator.cost(terrain_id, base_cost) {
                Some(c) => c,
                None => continue,
            };

            if cost > remaining {
                continue;
            }

            let new_remaining = remaining - cost;

            // 检查占用（自身位置除外）
            let is_blocked = match moving_entity {
                Some(entity) => occupancy.is_occupied_except(next, entity),
                None => occupancy.is_occupied(next),
            };
            if is_blocked {
                continue;
            }

            if let Some(&prev_remaining) = reachable.get(&next) {
                if prev_remaining >= new_remaining {
                    continue;
                }
            }

            reachable.insert(next, new_remaining);
            queue.push_back((next, new_remaining));
        }
    }

    reachable.remove(&start);
    reachable
}
