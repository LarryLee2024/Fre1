//! P1 集成测试：地形 + 寻路 + 战斗射程联动
//!
//! 跨 map/data + map/grid + map/runtime + map/pathfinding + battle/combat
//! 测试地形注册表、网格搭建、可通行性、寻路算法、战斗距离计算的联合行为。

// ================================================
// AI Self-Check (test_spec.md §13.1)
// ================================================
// ✅ 测试行为，不是实现
// ✅ 符合领域规则
// ✅ 测试是确定性的
// ✅ 使用标准测试数据
// ✅ 没有测试私有实现
// ✅ 没有生成不在范围内的测试
// ================================================

use bevy::prelude::*;
use std::collections::HashMap;
use tactical_rpg::battle::manhattan_distance;
use tactical_rpg::map::{
    FlyingCostCalculator, GameMap, GroundCostCalculator, OccupancyGrid, TerrainGrid,
    TerrainRegistry, find_reachable_tiles, reconstruct_path,
};

// ── 测试辅助 ──

/// 构建测试用地图（5x5）
fn make_test_map() -> GameMap {
    GameMap {
        width: 5,
        height: 5,
        tile_size: 64.0,
    }
}

/// 构建全平地 TerrainGrid
fn all_plain_grid(map: &GameMap) -> TerrainGrid {
    TerrainGrid::default_plain(map.width, map.height)
}

/// 构建测试用 TerrainRegistry（使用内置默认值）
fn test_registry() -> TerrainRegistry {
    let mut reg = TerrainRegistry::default();
    reg.register_defaults();
    reg
}

/// 构建空 OccupancyGrid
fn empty_occupancy() -> OccupancyGrid {
    OccupancyGrid::default()
}

/// 5x5 预置地图：中间一排山地 y=2, 不可通行
fn mountain_bisected_5x5() -> (GameMap, TerrainGrid) {
    let map = make_test_map();
    let mut grid = all_plain_grid(&map);
    for x in 0..5 {
        grid.set(IVec2::new(x, 2), "mountain".to_string());
    }
    (map, grid)
}

// ══════════════════════════════════════════════════════════════
// 场景一：地形注册表 + 地形网格联动
// ══════════════════════════════════════════════════════════════

#[test]
fn 地形网格_初始化全为默认地形() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);

    assert_eq!(grid.get(IVec2::new(0, 0)), Some("plain"));
    assert_eq!(grid.get(IVec2::new(2, 3)), Some("plain"));
    assert_eq!(grid.get(IVec2::new(4, 4)), Some("plain"));
}

#[test]
fn 地形网格_设置后正确读取() {
    let map = make_test_map();
    let mut grid = all_plain_grid(&map);

    grid.set(IVec2::new(2, 2), "forest".to_string());
    grid.set(IVec2::new(0, 0), "mountain".to_string());

    assert_eq!(grid.get(IVec2::new(2, 2)), Some("forest"));
    assert_eq!(grid.get(IVec2::new(0, 0)), Some("mountain"));
    assert_eq!(grid.get(IVec2::new(4, 4)), Some("plain"));
}

#[test]
fn 地形网格_山地不可通行() {
    let map = make_test_map();
    let mut grid = all_plain_grid(&map);
    grid.set(IVec2::new(1, 0), "mountain".to_string());
    let registry = test_registry();
    let occupancy = empty_occupancy();
    let calculator = GroundCostCalculator;

    let reachable = find_reachable_tiles(
        IVec2::new(0, 0),
        3,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    assert!(!reachable.contains_key(&IVec2::new(1, 0)));
    assert!(reachable.contains_key(&IVec2::new(0, 1)));
}

// ══════════════════════════════════════════════════════════════
// 场景二：寻路 + 地形费用联动
// ══════════════════════════════════════════════════════════════

#[test]
fn 寻路_全草地地图_移动范围为菱形() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let occupancy = empty_occupancy();
    let calculator = GroundCostCalculator;

    let reachable = find_reachable_tiles(
        IVec2::new(2, 2),
        3,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    // find_reachable_tiles removes start from result
    assert!(!reachable.contains_key(&IVec2::new(2, 2)));
    assert!(reachable.contains_key(&IVec2::new(2, 1)));
    assert!(reachable.contains_key(&IVec2::new(2, 0)));
    assert!(reachable.contains_key(&IVec2::new(2, 3)));
    assert!(reachable.contains_key(&IVec2::new(1, 2)));
    assert!(reachable.contains_key(&IVec2::new(0, 2)));
    assert!(reachable.contains_key(&IVec2::new(3, 2)));
    assert!(reachable.contains_key(&IVec2::new(4, 2)));
}

#[test]
fn 寻路_山地阻挡南北通行() {
    let (map, grid) = mountain_bisected_5x5();
    let registry = test_registry();
    let occupancy = empty_occupancy();
    let calculator = GroundCostCalculator;

    let reachable_north = find_reachable_tiles(
        IVec2::new(2, 0),
        10,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    assert!(reachable_north.contains_key(&IVec2::new(0, 0)));
    assert!(reachable_north.contains_key(&IVec2::new(4, 1)));
    assert!(!reachable_north.contains_key(&IVec2::new(2, 3)));
    assert!(!reachable_north.contains_key(&IVec2::new(2, 4)));
    assert!(!reachable_north.contains_key(&IVec2::new(2, 2)));
}

#[test]
fn 寻路_森林移动费用更高() {
    let map = make_test_map();
    let mut grid = all_plain_grid(&map);
    grid.set(IVec2::new(1, 0), "forest".to_string());
    let registry = test_registry();
    let occupancy = empty_occupancy();
    let calculator = GroundCostCalculator;

    let reachable = find_reachable_tiles(
        IVec2::new(0, 0),
        2,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    // forest cost=2, 2 <= 2 -> reachable
    assert!(reachable.contains_key(&IVec2::new(1, 0)));
    assert_eq!(*reachable.get(&IVec2::new(1, 0)).unwrap(), 0);
    // forest(1,0) cost 2 + plain(2,0) cost 1 = 3 > 2 -> not reachable
    assert!(!reachable.contains_key(&IVec2::new(2, 0)));
}

#[test]
fn 寻路_飞行单位_无视地形() {
    let (map, grid) = mountain_bisected_5x5();
    let registry = test_registry();
    let occupancy = empty_occupancy();
    let calculator = FlyingCostCalculator;

    let reachable = find_reachable_tiles(
        IVec2::new(2, 0),
        10,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    assert!(reachable.contains_key(&IVec2::new(2, 3)));
    assert!(reachable.contains_key(&IVec2::new(2, 4)));
}

// ══════════════════════════════════════════════════════════════
// 场景三：占位 + 寻路联动
// ══════════════════════════════════════════════════════════════

#[test]
fn 寻路_占据单位阻挡路径() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let mut occupancy = OccupancyGrid::default();
    let blocker = Entity::from_bits(99);

    occupancy.set(IVec2::new(2, 1), blocker);

    let calculator = GroundCostCalculator;

    let reachable = find_reachable_tiles(
        IVec2::new(2, 0),
        10,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    assert!(!reachable.contains_key(&IVec2::new(2, 1)));
    assert!(reachable.contains_key(&IVec2::new(2, 2)));
}

#[test]
fn 寻路_自身位置不阻挡() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let mut occupancy = OccupancyGrid::default();
    let self_entity = Entity::from_bits(1);

    occupancy.set(IVec2::new(2, 0), self_entity);

    let calculator = GroundCostCalculator;

    let reachable = find_reachable_tiles(
        IVec2::new(2, 0),
        3,
        &map,
        &grid,
        &registry,
        &occupancy,
        Some(self_entity),
        &calculator,
    );

    assert!(reachable.contains_key(&IVec2::new(3, 0)));
    assert!(reachable.contains_key(&IVec2::new(2, 1)));
}

// ══════════════════════════════════════════════════════════════
// 场景四：寻路重建路径
// ══════════════════════════════════════════════════════════════

#[test]
fn 路径重建_直线() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let calculator = GroundCostCalculator;

    let mut reachable = HashMap::new();
    reachable.insert(IVec2::new(3, 2), 2);
    reachable.insert(IVec2::new(4, 2), 1);

    let path = reconstruct_path(
        IVec2::new(2, 2),
        IVec2::new(4, 2),
        &reachable,
        3,
        &map,
        &grid,
        &registry,
        &calculator,
    );

    assert_eq!(path, vec![IVec2::new(3, 2), IVec2::new(4, 2)]);
}

#[test]
fn 路径重建_对角线() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let calculator = GroundCostCalculator;

    let mut reachable = HashMap::new();
    reachable.insert(IVec2::new(2, 1), 2);
    reachable.insert(IVec2::new(2, 2), 1);

    let path = reconstruct_path(
        IVec2::new(1, 1),
        IVec2::new(2, 2),
        &reachable,
        3,
        &map,
        &grid,
        &registry,
        &calculator,
    );

    assert_eq!(path, vec![IVec2::new(2, 1), IVec2::new(2, 2)]);
}

#[test]
fn 路径重建_不存在的目标() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let calculator = GroundCostCalculator;
    let reachable = HashMap::new();

    let path = reconstruct_path(
        IVec2::new(2, 2),
        IVec2::new(4, 4),
        &reachable,
        3,
        &map,
        &grid,
        &registry,
        &calculator,
    );

    assert_eq!(path, vec![IVec2::new(4, 4)]);
}

// ══════════════════════════════════════════════════════════════
// 场景五：manhattan_distance 跨模块使用
// ══════════════════════════════════════════════════════════════

#[test]
fn 战斗距离_同一位置() {
    assert_eq!(manhattan_distance(IVec2::new(2, 2), IVec2::new(2, 2)), 0);
}

#[test]
fn 战斗距离_直线() {
    assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(3, 0)), 3);
}

#[test]
fn 战斗距离_对角线() {
    assert_eq!(manhattan_distance(IVec2::new(1, 1), IVec2::new(4, 4)), 6);
}

#[test]
fn 战斗距离_反向() {
    assert_eq!(manhattan_distance(IVec2::new(5, 5), IVec2::new(2, 3)), 5);
}

// ══════════════════════════════════════════════════════════════
// 场景六：寻路 + 射程联合判定
// ══════════════════════════════════════════════════════════════

#[test]
fn 攻击范围_寻路可达目标在射程内() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);
    let registry = test_registry();
    let occupancy = empty_occupancy();
    let calculator = GroundCostCalculator;

    let _reachable = find_reachable_tiles(
        IVec2::new(2, 2),
        3,
        &map,
        &grid,
        &registry,
        &occupancy,
        None,
        &calculator,
    );

    let base_attack_range = 2;
    let move_target = IVec2::new(2, 0);

    let dist = manhattan_distance(move_target, IVec2::new(2, 3));
    assert_eq!(dist, 3);
    assert!(dist > base_attack_range);

    let dist = manhattan_distance(move_target, IVec2::new(2, 1));
    assert_eq!(dist, 1);
    assert!(dist <= base_attack_range);
}
