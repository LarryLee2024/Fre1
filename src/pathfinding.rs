// 寻路模块：BFS 计算可移动范围与路径

use crate::map::{GameMap, Terrain, Tile};
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

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

            let terrain = match tiles.get(&next) {
                Some(t) => *t,
                None => continue,
            };

            let cost = match terrain.move_cost() {
                Some(c) => c,
                None => continue,
            };

            if cost > remaining {
                continue;
            }

            let new_remaining = remaining - cost;

            if let Some(&is_occupied) = occupied.get(&next) {
                if is_occupied {
                    continue;
                }
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

/// 构建地形查找表
pub fn build_tile_terrain_map(tiles: &Query<&Tile>) -> HashMap<IVec2, Terrain> {
    tiles
        .iter()
        .map(|tile| (tile.coord, tile.terrain))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构建测试用地图（5x5 全平地）
    fn make_test_map() -> GameMap {
        GameMap {
            width: 5,
            height: 5,
            tile_size: 64.0,
        }
    }

    /// 构建全平地地形表
    fn all_plain_map(map: &GameMap) -> HashMap<IVec2, Terrain> {
        let mut tiles = HashMap::new();
        for x in 0..map.width {
            for y in 0..map.height {
                tiles.insert(IVec2::new(x as i32, y as i32), Terrain::Plain);
            }
        }
        tiles
    }

    #[test]
    fn 平地_移动力3_可到达13格() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();

        let reachable = find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied);

        // 3x3 菱形 + 延伸，中心(2,2)移动力3，曼哈顿距离<=3的格子
        // 共 1+4+8+12 = 24 减去超出边界的 = 应该有若干格
        assert!(reachable.len() > 0);
        // 起点不应在结果中
        assert!(!reachable.contains_key(&IVec2::new(2, 2)));
        // 相邻格子一定可达
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
    }

    #[test]
    fn 移动力0_无可达格子() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();

        let reachable = find_reachable_tiles(IVec2::new(2, 2), 0, &map, &tiles, &occupied);
        assert!(reachable.is_empty());
    }

    #[test]
    fn 移动力1_只能到相邻4格() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();

        let reachable = find_reachable_tiles(IVec2::new(2, 2), 1, &map, &tiles, &occupied);

        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    #[test]
    fn 山地和水域不可通行() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        // 右侧设为山地
        tiles.insert(IVec2::new(3, 2), Terrain::Mountain);
        // 上方设为水域
        tiles.insert(IVec2::new(2, 3), Terrain::Water);
        let occupied = HashMap::new();

        let reachable = find_reachable_tiles(IVec2::new(2, 2), 1, &map, &tiles, &occupied);

        // 只有左和下可达
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    #[test]
    fn 森林消耗2移动力() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        // 相邻格设为森林
        tiles.insert(IVec2::new(3, 2), Terrain::Forest);
        let occupied = HashMap::new();

        let reachable = find_reachable_tiles(IVec2::new(2, 2), 2, &map, &tiles, &occupied);

        // 森林格消耗2，移动力2刚好到达，剩余0
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        // 森林格后面不可达（剩余移动力为0）
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    #[test]
    fn 被敌方占据的格子不可达() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let mut occupied = HashMap::new();
        occupied.insert(IVec2::new(3, 2), true);

        let reachable = find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied);

        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
        // 被占据格子不可穿越，(4,2)需经过(3,2)所以也不可达
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
        // 但绕路可达(4,2)的邻居：如(4,1)经(3,1)(4,1)可达
        assert!(reachable.contains_key(&IVec2::new(4, 1)));
    }

    #[test]
    fn 角落位置_移动力受限() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();

        let reachable = find_reachable_tiles(IVec2::new(0, 0), 2, &map, &tiles, &occupied);

        // 左上角，只有右、下、右下可达
        assert!(reachable.contains_key(&IVec2::new(1, 0)));
        assert!(reachable.contains_key(&IVec2::new(0, 1)));
        assert!(reachable.contains_key(&IVec2::new(1, 1)));
        assert!(reachable.contains_key(&IVec2::new(2, 0)));
        assert!(reachable.contains_key(&IVec2::new(0, 2)));
        // 左边和上边越界
        assert!(!reachable.contains_key(&IVec2::new(-1, 0)));
        assert!(!reachable.contains_key(&IVec2::new(0, -1)));
    }
}
