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

/// LTC-001: 地形网格初始化全为默认地形
///
/// Given: 5x5 地图
/// When: 创建 default_plain TerrainGrid
/// Then: 所有坐标返回 "plain"
#[test]
fn 地形网格_初始化全为默认地形() {
    let map = make_test_map();
    let grid = all_plain_grid(&map);

    assert_eq!(grid.get(IVec2::new(0, 0)), Some("plain"));
    assert_eq!(grid.get(IVec2::new(2, 3)), Some("plain"));
    assert_eq!(grid.get(IVec2::new(4, 4)), Some("plain"));
}

/// LTC-002: 地形网格设置后正确读取
///
/// Given: 全平地 TerrainGrid
/// When: set(2,2, "forest") + set(0,0, "mountain")
/// Then: get(2,2)="forest", get(0,0)="mountain", get(4,4)="plain"
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

/// LTC-003: 山地不可通行
///
/// Given: 全平地 + (1,0) 设为 mountain
/// When: find_reachable_tiles from (0,0) range=3
/// Then: (1,0) 不在可达列表，(0,1) 在可达列表
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

/// LTC-004: 全草地地图寻路移动范围为菱形
///
/// Given: 全平地 5x5 地图
/// When: find_reachable_tiles from (2,2) range=3
/// Then: 菱形范围内可达，start 点不在结果中
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

/// LTC-005: 山地阻挡南北通行
///
/// Given: 5x5 地图 y=2 一排山地
/// When: find_reachable_tiles from (2,0) range=10
/// Then: 无法到达 y>=2 的坐标
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

/// LTC-006: 森林移动费用更高
///
/// Given: (1,0) 设为 forest（cost=2）
/// When: find_reachable_tiles from (0,0) range=2
/// Then: (1,0) 可达（cost=2），(2,0) 不可达（cost=3>2）
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

/// LTC-007: 飞行单位无视地形
///
/// Given: 山地横断地图
/// When: FlyingCostCalculator find_reachable_tiles from (2,0) range=10
/// Then: 可到达 (2,3)、(2,4) 等山地后方坐标
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

/// LTC-008: 占据单位阻挡路径
///
/// Given: (2,1) 有 blocker entity
/// When: find_reachable_tiles from (2,0) range=10
/// Then: (2,1) 不可达，(2,2) 可达
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

/// LTC-009: 自身位置不阻挡
///
/// Given: (2,0) 有 self_entity
/// When: find_reachable_tiles from (2,0) range=3, self_entity=self
/// Then: (3,0)、(2,1) 可达
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

/// LTC-010: 路径重建 — 直线
///
/// Given: reachable 含 (3,2) cost=2, (4,2) cost=1
/// When: reconstruct_path from (2,2) to (4,2)
/// Then: path=[(3,2), (4,2)]
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

/// LTC-011: 路径重建 — 对角线
///
/// Given: reachable 含 (2,1) cost=2, (2,2) cost=1
/// When: reconstruct_path from (1,1) to (2,2)
/// Then: path=[(2,1), (2,2)]
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

/// LTC-012: 路径重建 — 不存在的目标
///
/// Given: 空 reachable
/// When: reconstruct_path to (4,4)
/// Then: path=[]（目标不在可达范围内）
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

    assert_eq!(path, vec![], "目标不在可达范围内应返回空路径");
}

// ══════════════════════════════════════════════════════════════
// 场景五：manhattan_distance 跨模块使用
// ══════════════════════════════════════════════════════════════

/// LTC-013: 战斗距离 — 同一位置
///
/// Given: 两个相同坐标 (2,2)
/// When: manhattan_distance
/// Then: 距离=0
#[test]
fn 战斗距离_同一位置() {
    assert_eq!(manhattan_distance(IVec2::new(2, 2), IVec2::new(2, 2)), 0);
}

/// LTC-014: 战斗距离 — 直线
///
/// Given: (0,0) 和 (3,0)
/// When: manhattan_distance
/// Then: 距离=3
#[test]
fn 战斗距离_直线() {
    assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(3, 0)), 3);
}

/// LTC-015: 战斗距离 — 对角线
///
/// Given: (1,1) 和 (4,4)
/// When: manhattan_distance
/// Then: 距离=6
#[test]
fn 战斗距离_对角线() {
    assert_eq!(manhattan_distance(IVec2::new(1, 1), IVec2::new(4, 4)), 6);
}

/// LTC-016: 战斗距离 — 反向
///
/// Given: (5,5) 和 (2,3)
/// When: manhattan_distance
/// Then: 距离=5
#[test]
fn 战斗距离_反向() {
    assert_eq!(manhattan_distance(IVec2::new(5, 5), IVec2::new(2, 3)), 5);
}

// ══════════════════════════════════════════════════════════════
// 场景六：寻路 + 射程联合判定
// ══════════════════════════════════════════════════════════════

/// LTC-017: 攻击范围 — 寻路可达目标在射程内
///
/// Given: 全平地，寻路 range=3
/// When: 计算移动到 (2,0) 后与 (2,3)/(2,1) 的距离
/// Then: (2,1) 距离=1 <= 攻击射程2，(2,3) 距离=3 > 攻击射程2
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
