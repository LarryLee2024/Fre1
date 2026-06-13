// 寻路模块：BFS 计算可移动范围与路径
// TerrainCostCalculator trait 使用 terrain_id: &str，支持数据驱动扩展
// 寻路直接从 TerrainGrid + OccupancyGrid 读取，不再依赖 Tile Entity

mod algorithms; // BFS 移动范围与路径搜索算法
mod cost; // TerrainCostRegistry 地形消耗注册表

pub use algorithms::*;
pub use cost::*;

#[cfg(test)]
mod tests {
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

    use super::*;
    use crate::core::tag::{GameplayTag, GameplayTags};
    use crate::map::data::TerrainRegistry;
    use crate::map::grid::GameMap;
    use crate::map::runtime::{OccupancyGrid, TerrainGrid};
    use bevy::prelude::*;
    use std::collections::HashMap;

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

    /// 构建测试用 TerrainRegistry
    fn test_registry() -> TerrainRegistry {
        let mut reg = TerrainRegistry::default();
        reg.register_defaults();
        reg
    }

    /// 构建空 OccupancyGrid
    fn empty_occupancy() -> OccupancyGrid {
        OccupancyGrid::default()
    }

    // ── 步兵（GroundCostCalculator）测试 ──

    /// Test ID: MAP-PF-001
    /// Title: 步兵平地 MP=3 可达范围
    ///
    /// Given: 5x5 全平地地图，步兵在 (2,2)，MP=3
    /// When: 调用 find_reachable_tiles
    /// Then: 返回可达格子集合，起点不在集合内
    ///
    /// Assertions: 非空，(3,2) 和 (2,3) 可达，(2,2) 不可达
    #[test]
    fn ground_unit_reachability_with_mp_3_on_plains() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

        // When
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

        // Then
        assert!(!reachable.is_empty());
        assert!(!reachable.contains_key(&IVec2::new(2, 2)));
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
    }

    /// Test ID: MAP-PF-002
    /// Title: 步兵 MP=0 无可达格子
    ///
    /// Given: 5x5 全平地地图，步兵在 (2,2)，MP=0
    /// When: 调用 find_reachable_tiles
    /// Then: 返回空集合
    ///
    /// Assertions: reachable.is_empty()
    #[test]
    fn ground_unit_mp_0_no_reachable_tiles() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

        // When
        let reachable = find_reachable_tiles(
            IVec2::new(2, 2),
            0,
            &map,
            &grid,
            &registry,
            &occupancy,
            None,
            &calculator,
        );

        // Then
        assert!(reachable.is_empty());
    }

    /// Test ID: MAP-PF-003
    /// Title: 步兵 MP=1 仅可达相邻 4 格
    ///
    /// Given: 5x5 全平地地图，步兵在 (2,2)，MP=1
    /// When: 调用 find_reachable_tiles
    /// Then: 返回 4 个相邻格子
    ///
    /// Assertions: len==4，包含上下左右
    #[test]
    fn ground_unit_mp_1_adjacent_4_tiles_only() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

        // When
        let reachable = find_reachable_tiles(
            IVec2::new(2, 2),
            1,
            &map,
            &grid,
            &registry,
            &occupancy,
            None,
            &calculator,
        );

        // Then
        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    /// Test ID: MAP-PF-004
    /// Title: 步兵不可通行山地和水域
    ///
    /// Given: 5x5 地图，(3,2)=mountain, (2,3)=water，MP=1
    /// When: 调用 find_reachable_tiles
    /// Then: mountain 和 water 不可达
    ///
    /// Assertions: len==2，仅 (1,2) 和 (2,1) 可达
    #[test]
    fn ground_unit_cannot_traverse_mountain_or_water() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        grid.set(IVec2::new(2, 3), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

        // When
        let reachable = find_reachable_tiles(
            IVec2::new(2, 2),
            1,
            &map,
            &grid,
            &registry,
            &occupancy,
            None,
            &calculator,
        );

        // Then
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    /// Test ID: MAP-PF-005
    /// Title: 步兵森林消耗 2 MP
    ///
    /// Given: 5x5 地图，(3,2)=forest，MP=2
    /// When: 调用 find_reachable_tiles
    /// Then: (3,2) 剩余 MP=0 可达，(4,2) 不可达
    ///
    /// Assertions: (3,2) 在集合中且 remaining=0，(4,2) 不在集合中
    #[test]
    fn ground_unit_forest_costs_2_mp() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "forest".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

        // When
        let reachable = find_reachable_tiles(
            IVec2::new(2, 2),
            2,
            &map,
            &grid,
            &registry,
            &occupancy,
            None,
            &calculator,
        );

        // Then
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    /// Test ID: MAP-PF-006
    /// Title: 步兵被占据格子不可达
    ///
    /// Given: 5x5 地图，(3,2) 被 blocker 占据，MP=3
    /// When: 调用 find_reachable_tiles
    /// Then: (3,2) 不可达，绕过障碍后 (4,1) 仍可达
    ///
    /// Assertions: (3,2) 不可达，(4,2) 不可达，(4,1) 可达
    #[test]
    fn ground_unit_occupied_tile_unreachable() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let mut occupancy = OccupancyGrid::default();
        let blocker = Entity::from_bits(99);
        occupancy.set(IVec2::new(3, 2), blocker);
        let calculator = GroundCostCalculator;

        // When
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

        // Then
        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
        assert!(reachable.contains_key(&IVec2::new(4, 1)));
    }

    /// Test ID: MAP-PF-007
    /// Title: 步兵自身位置不算被占用
    ///
    /// Given: 5x5 地图，(2,2) 被 self_entity 占据，MP=1
    /// When: 调用 find_reachable_tiles（传入 self_entity）
    /// Then: 仍可达相邻 4 格
    ///
    /// Assertions: len==4
    #[test]
    fn ground_unit_self_position_not_counted_as_occupied() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let mut occupancy = OccupancyGrid::default();
        let self_entity = Entity::from_bits(1);
        occupancy.set(IVec2::new(2, 2), self_entity);
        let calculator = GroundCostCalculator;

        // When
        let reachable = find_reachable_tiles(
            IVec2::new(2, 2),
            1,
            &map,
            &grid,
            &registry,
            &occupancy,
            Some(self_entity),
            &calculator,
        );

        // Then
        assert_eq!(reachable.len(), 4);
    }

    /// Test ID: MAP-PF-008
    /// Title: 步兵角落位置 MP 受限
    ///
    /// Given: 5x5 全平地地图，步兵在 (0,0)，MP=2
    /// When: 调用 find_reachable_tiles
    /// Then: 不可达负坐标
    ///
    /// Assertions: (1,0) 和 (0,1) 可达，(-1,0) 和 (0,-1) 不可达
    #[test]
    fn ground_unit_corner_position_mp_limited() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

        // When
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

        // Then
        assert!(reachable.contains_key(&IVec2::new(1, 0)));
        assert!(reachable.contains_key(&IVec2::new(0, 1)));
        assert!(!reachable.contains_key(&IVec2::new(-1, 0)));
        assert!(!reachable.contains_key(&IVec2::new(0, -1)));
    }

    // ── 飞行（FlyingCostCalculator）测试 ──

    /// Test ID: MAP-PF-009
    /// Title: 飞行单位山地和水域可通行，成本为 1
    ///
    /// Given: 5x5 地图，(3,2)=mountain, (2,3)=water，MP=1
    /// When: 调用 find_reachable_tiles（FlyingCostCalculator）
    /// Then: mountain 和 water 可达，共 4 格
    ///
    /// Assertions: len==4，(3,2) 和 (2,3) 可达
    #[test]
    fn flying_unit_traverses_mountain_water_cost_1() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        grid.set(IVec2::new(2, 3), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = FlyingCostCalculator;

        // When
        let reachable = find_reachable_tiles(
            IVec2::new(2, 2),
            1,
            &map,
            &grid,
            &registry,
            &occupancy,
            None,
            &calculator,
        );

        // Then
        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
    }

    // ── 骑兵（MountedCostCalculator）测试 ──

    /// Test ID: MAP-PF-010
    /// Title: 骑兵平原成本 1，森林成本 3
    ///
    /// Given: 5x5 地图，(3,2)=forest，MP=3
    /// When: 调用 find_reachable_tiles（MountedCostCalculator）
    /// Then: forest 剩余 MP=0，(4,2) 不可达
    ///
    /// Assertions: (3,2) remaining=0，(4,2) 不可达
    #[test]
    fn mounted_unit_plains_cost_1_forest_cost_3() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "forest".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = MountedCostCalculator;

        // When
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

        // Then
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    /// Test ID: MAP-PF-011
    /// Title: 骑兵山地和水域不可通行
    ///
    /// Given: 5x5 地图，(3,2)=mountain, (2,3)=water，MP=3
    /// When: 调用 find_reachable_tiles（MountedCostCalculator）
    /// Then: mountain 和 water 不可达
    ///
    /// Assertions: (3,2) 和 (2,3) 不可达
    #[test]
    fn mounted_unit_cannot_traverse_mountain_or_water() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        grid.set(IVec2::new(2, 3), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = MountedCostCalculator;

        // When
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

        // Then
        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
        assert!(!reachable.contains_key(&IVec2::new(2, 3)));
    }

    // ── 水生（SwimmingCostCalculator）测试 ──

    /// Test ID: MAP-PF-012
    /// Title: 水生单位水域成本 1
    ///
    /// Given: 5x5 地图，(3,2)=water，MP=3
    /// When: 调用 find_reachable_tiles（SwimmingCostCalculator）
    /// Then: water 可达
    ///
    /// Assertions: (3,2) 可达
    #[test]
    fn swimming_unit_water_cost_1() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = SwimmingCostCalculator;

        // When
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

        // Then
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
    }

    /// Test ID: MAP-PF-013
    /// Title: 水生单位山地不可通行
    ///
    /// Given: 5x5 地图，(3,2)=mountain，MP=3
    /// When: 调用 find_reachable_tiles（SwimmingCostCalculator）
    /// Then: mountain 不可达
    ///
    /// Assertions: (3,2) 不可达
    #[test]
    fn swimming_unit_cannot_traverse_mountain() {
        // Given
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = SwimmingCostCalculator;

        // When
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

        // Then
        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
    }

    // ── 注册表测试 ──

    /// Test ID: MAP-PF-014
    /// Title: TerrainCostRegistry 默认包含 4 种计算器
    ///
    /// Given: 默认 TerrainCostRegistry
    /// When: 查询 ground/flying/mounted/swimming
    /// Then: 全部返回 Some
    ///
    /// Assertions: 四个计算器都存在
    #[test]
    fn terrain_cost_registry_contains_four_calculators_by_default() {
        // Given
        let registry = TerrainCostRegistry::default();

        // When & Then
        assert!(registry.get("ground").is_some());
        assert!(registry.get("flying").is_some());
        assert!(registry.get("mounted").is_some());
        assert!(registry.get("swimming").is_some());
    }

    /// Test ID: MAP-PF-015
    /// Title: TerrainCostRegistry 根据标签解析计算器
    ///
    /// Given: 默认 TerrainCostRegistry
    /// When: 使用不同 GameplayTags 解析
    /// Then: 默认返回 ground，FLYING 返回 flying，MOUNTED 返回 mounted，SWIMMING 返回 swimming
    ///
    /// Assertions: 四种标签分别解析出正确计算器
    #[test]
    fn terrain_cost_registry_resolves_from_tags() {
        // Given
        let registry = TerrainCostRegistry::default();

        // When & Then - ground
        let tags = GameplayTags::default();
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "ground");

        // When & Then - flying
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FLYING);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "flying");

        // When & Then - mounted
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::MOUNTED);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "mounted");

        // When & Then - swimming
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::SWIMMING);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "swimming");
    }

    /// Test ID: MAP-PF-016
    /// Title: 水生优先级高于飞行
    ///
    /// Given: 同时有 FLYING 和 SWIMMING 标签
    /// When: 调用 resolve_from_tags
    /// Then: 返回 swimming
    ///
    /// Assertions: calc.name() == "swimming"
    #[test]
    fn swimming_priority_over_flying() {
        // Given
        let registry = TerrainCostRegistry::default();
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FLYING);
        tags.add(GameplayTag::SWIMMING);

        // When
        let calc = registry.resolve_from_tags(&tags);

        // Then
        assert_eq!(calc.name(), "swimming");
    }

    // ── reconstruct_path 测试 ──

    /// Test ID: MAP-PF-017
    /// Title: 回溯路径 - 起点即终点返回自身
    ///
    /// Given: 5x5 地图，start == end == (2,2)
    /// When: 调用 reconstruct_path
    /// Then: 返回 vec![(2,2)]
    ///
    /// Assertions: path == vec![IVec2::new(2,2)]
    #[test]
    fn reconstruct_path_same_start_end_returns_self() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let reachable = HashMap::new();
        let calculator = GroundCostCalculator;

        // When
        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(2, 2),
            &reachable,
            3,
            &map,
            &grid,
            &registry,
            &calculator,
        );

        // Then
        assert_eq!(path, vec![IVec2::new(2, 2)]);
    }

    /// Test ID: MAP-PF-018
    /// Title: reconstruct_path - 回溯路径 L 形
    ///
    /// Given: 5x5 地图，reachable 包含 (2,1) 和 (2,2)
    /// When: 从 (1,1) 回溯到 (2,2)
    /// Then: 返回 L 形路径 [(2,1), (2,2)]
    ///
    /// Assertions: path == vec![(2,1), (2,2)]
    #[test]
    fn reconstruct_path_l_shape() {
        // Given
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let calculator = GroundCostCalculator;

        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 1), 2);
        reachable.insert(IVec2::new(2, 2), 1);

        // When
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

        // Then
        assert_eq!(path, vec![IVec2::new(2, 1), IVec2::new(2, 2)]);
    }
}
