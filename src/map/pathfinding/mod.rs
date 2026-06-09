// 寻路模块：BFS 计算可移动范围与路径
// TerrainCostCalculator trait 使用 terrain_id: &str，支持数据驱动扩展
// 寻路直接从 TerrainGrid + OccupancyGrid 读取，不再依赖 Tile Entity

mod algorithms;
mod cost;

pub use algorithms::*;
pub use cost::*;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::collections::HashMap;
    use crate::core::tag::{GameplayTag, GameplayTags};
    use crate::map::data::TerrainRegistry;
    use crate::map::grid::GameMap;
    use crate::map::runtime::{OccupancyGrid, TerrainGrid};

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

    #[test]
    fn 步兵_平地_移动力3_可达() {
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

        assert!(!reachable.is_empty());
        assert!(!reachable.contains_key(&IVec2::new(2, 2)));
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
    }

    #[test]
    fn 步兵_移动力0_无可达格子() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

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
        assert!(reachable.is_empty());
    }

    #[test]
    fn 步兵_移动力1_只能到相邻4格() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

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

        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    #[test]
    fn 步兵_山地和水域不可通行() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        grid.set(IVec2::new(2, 3), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

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

        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    #[test]
    fn 步兵_森林消耗2移动力() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "forest".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = GroundCostCalculator;

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

        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    #[test]
    fn 步兵_被占据的格子不可达() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let mut occupancy = OccupancyGrid::default();
        let blocker = Entity::from_bits(99);
        occupancy.set(IVec2::new(3, 2), blocker);
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

        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
        assert!(reachable.contains_key(&IVec2::new(4, 1)));
    }

    #[test]
    fn 步兵_自身位置不算被占用() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let mut occupancy = OccupancyGrid::default();
        let self_entity = Entity::from_bits(1);
        occupancy.set(IVec2::new(2, 2), self_entity);
        let calculator = GroundCostCalculator;

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

        assert_eq!(reachable.len(), 4);
    }

    #[test]
    fn 步兵_角落位置_移动力受限() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
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

        assert!(reachable.contains_key(&IVec2::new(1, 0)));
        assert!(reachable.contains_key(&IVec2::new(0, 1)));
        assert!(!reachable.contains_key(&IVec2::new(-1, 0)));
        assert!(!reachable.contains_key(&IVec2::new(0, -1)));
    }

    // ── 飞行（FlyingCostCalculator）测试 ──

    #[test]
    fn 飞行_山地和水域可通行_成本为1() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        grid.set(IVec2::new(2, 3), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = FlyingCostCalculator;

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

        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
    }

    // ── 骑兵（MountedCostCalculator）测试 ──

    #[test]
    fn 骑兵_平原成本1_森林成本3() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "forest".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = MountedCostCalculator;

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

        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    #[test]
    fn 骑兵_山地和水域不可通行() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        grid.set(IVec2::new(2, 3), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = MountedCostCalculator;

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

        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
        assert!(!reachable.contains_key(&IVec2::new(2, 3)));
    }

    // ── 水生（SwimmingCostCalculator）测试 ──

    #[test]
    fn 水生_水域成本1() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "water".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = SwimmingCostCalculator;

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

        assert!(reachable.contains_key(&IVec2::new(3, 2)));
    }

    #[test]
    fn 水生_山地不可通行() {
        let map = make_test_map();
        let mut grid = all_plain_grid(&map);
        grid.set(IVec2::new(3, 2), "mountain".to_string());
        let registry = test_registry();
        let occupancy = empty_occupancy();
        let calculator = SwimmingCostCalculator;

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

        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
    }

    // ── 注册表测试 ──

    #[test]
    fn 注册表_默认包含四种计算器() {
        let registry = TerrainCostRegistry::default();
        assert!(registry.get("ground").is_some());
        assert!(registry.get("flying").is_some());
        assert!(registry.get("mounted").is_some());
        assert!(registry.get("swimming").is_some());
    }

    #[test]
    fn 注册表_根据标签解析计算器() {
        let registry = TerrainCostRegistry::default();

        let tags = GameplayTags::default();
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "ground");

        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FLYING);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "flying");

        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::MOUNTED);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "mounted");

        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::SWIMMING);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "swimming");
    }

    #[test]
    fn 注册表_水生优先级高于飞行() {
        let registry = TerrainCostRegistry::default();
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FLYING);
        tags.add(GameplayTag::SWIMMING);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "swimming");
    }

    // ── reconstruct_path 测试 ──

    #[test]
    fn 回溯路径_同坐标返回目标() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let reachable = HashMap::new();
        let calculator = GroundCostCalculator;

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

        assert_eq!(path, vec![IVec2::new(2, 2)]);
    }

    #[test]
    fn 回溯路径_相邻格子() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let calculator = GroundCostCalculator;

        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(3, 2), 2);

        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(3, 2),
            &reachable,
            3,
            &map,
            &grid,
            &registry,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(3, 2)]);
    }

    #[test]
    fn 回溯路径_直线两格() {
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
    fn 回溯路径_不存在的目标() {
        let map = make_test_map();
        let grid = all_plain_grid(&map);
        let registry = test_registry();
        let reachable = HashMap::new();
        let calculator = GroundCostCalculator;

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

    #[test]
    fn 回溯路径_L形路径() {
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
}
