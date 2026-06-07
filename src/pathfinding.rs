// 寻路模块：BFS 计算可移动范围与路径

use std::collections::{HashMap, VecDeque};
use bevy::prelude::*;
use crate::map::{GameMap, Terrain, Tile};
use crate::unit::{Unit, GridPosition, Faction};

/// 寻路结果：可到达的格子及其剩余移动力
pub fn find_reachable_tiles(
    start: IVec2,
    move_points: u32,
    map: &GameMap,
    tiles: &HashMap<IVec2, Terrain>,
    occupied: &HashMap<IVec2, bool>,
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

            // 检查地形是否可通行
            let terrain = match tiles.get(&next) {
                Some(t) => *t,
                None => continue,
            };

            let cost = match terrain.move_cost() {
                Some(c) => c,
                None => continue, // 不可通行
            };

            if cost > remaining {
                continue;
            }

            let new_remaining = remaining - cost;

            // 不能穿过敌方单位，但可以穿过友方
            if let Some(&is_occupied) = occupied.get(&next) {
                if is_occupied {
                    continue;
                }
            }

            // 如果已有更优路径则跳过
            if let Some(&prev_remaining) = reachable.get(&next) {
                if prev_remaining >= new_remaining {
                    continue;
                }
            }

            reachable.insert(next, new_remaining);
            queue.push_back((next, new_remaining));
        }
    }

    reachable.remove(&start); // 起点不算可移动
    reachable
}

/// 构建地形查找表
pub fn build_tile_terrain_map(
    tiles: &Query<&Tile>,
) -> HashMap<IVec2, Terrain> {
    tiles.iter().map(|tile| (tile.coord, tile.terrain)).collect()
}

/// 构建单位占位表（敌方占位阻挡移动）
pub fn build_occupation_map(
    units: &Query<(&GridPosition, &Unit)>,
    current_faction: Faction,
) -> HashMap<IVec2, bool> {
    units
        .iter()
        .filter(|(_, unit)| unit.faction != current_faction)
        .map(|(pos, _)| (pos.coord, true))
        .collect()
}
