// 寻路模块：BFS 计算可移动范围与路径
// TerrainCostCalculator trait 使用 terrain_id: &str，支持数据驱动扩展
// 寻路直接从 TerrainGrid + OccupancyGrid 读取，不再依赖 Tile Entity

use super::data::TerrainRegistry;
use super::grid::GameMap;
use super::runtime::{OccupancyGrid, TerrainGrid};
use crate::core::tag::{GameplayTag, GameplayTags};
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

// ── 地形成本计算 trait ──

/// 地形移动成本计算规则 trait：描述不同单位类型的地形通行能力
/// terrain_id: 地形 ID 字符串（如 "plain", "forest"），由 TerrainRegistry 定义
/// base_cost: 从 TerrainRegistry 加载的基础成本（None 表示基础不可通行）
pub trait TerrainCostCalculator: Send + Sync + 'static {
    /// 计算器名称（用于注册和查找）
    fn name(&self) -> &'static str;
    /// 计算指定地形的移动成本，None 表示不可通行
    fn cost(&self, terrain_id: &str, base_cost: Option<u32>) -> Option<u32>;
}

// ── 内置实现 ──

/// 步兵成本计算器：使用基础成本
pub struct GroundCostCalculator;

impl TerrainCostCalculator for GroundCostCalculator {
    fn name(&self) -> &'static str {
        "ground"
    }

    fn cost(&self, _terrain_id: &str, base_cost: Option<u32>) -> Option<u32> {
        base_cost
    }
}

/// 飞行成本计算器：所有地形成本为1
pub struct FlyingCostCalculator;

impl TerrainCostCalculator for FlyingCostCalculator {
    fn name(&self) -> &'static str {
        "flying"
    }

    fn cost(&self, _terrain_id: &str, _base_cost: Option<u32>) -> Option<u32> {
        Some(1)
    }
}

/// 骑兵成本计算器：平原成本1，森林成本3，山地/水域不可通行
pub struct MountedCostCalculator;

impl TerrainCostCalculator for MountedCostCalculator {
    fn name(&self) -> &'static str {
        "mounted"
    }

    fn cost(&self, terrain_id: &str, _base_cost: Option<u32>) -> Option<u32> {
        match terrain_id {
            "plain" => Some(1),
            "forest" => Some(3),
            _ => None, // 骑兵无法进入山地和水域
        }
    }
}

/// 水生成本计算器：水域成本1，平原成本2，山地不可通行
pub struct SwimmingCostCalculator;

impl TerrainCostCalculator for SwimmingCostCalculator {
    fn name(&self) -> &'static str {
        "swimming"
    }

    fn cost(&self, terrain_id: &str, _base_cost: Option<u32>) -> Option<u32> {
        match terrain_id {
            "water" => Some(1),
            "plain" => Some(2),
            "forest" => Some(3),
            _ => None, // 水生单位无法进入山地
        }
    }
}

// ── 地形成本注册表 ──

/// 地形成本计算器注册表资源
#[derive(Resource)]
pub struct TerrainCostRegistry {
    calculators: HashMap<String, Box<dyn TerrainCostCalculator>>,
}

impl Default for TerrainCostRegistry {
    fn default() -> Self {
        let mut registry = Self {
            calculators: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl TerrainCostRegistry {
    /// 注册内置默认计算器
    fn register_defaults(&mut self) {
        self.register(Box::new(GroundCostCalculator));
        self.register(Box::new(FlyingCostCalculator));
        self.register(Box::new(MountedCostCalculator));
        self.register(Box::new(SwimmingCostCalculator));
    }

    /// 注册一个计算器
    pub fn register(&mut self, calculator: Box<dyn TerrainCostCalculator>) {
        self.calculators
            .insert(calculator.name().to_string(), calculator);
    }

    /// 按名称获取计算器
    pub fn get(&self, name: &str) -> Option<&dyn TerrainCostCalculator> {
        self.calculators.get(name).map(|c| c.as_ref())
    }

    /// 获取默认（步兵）计算器
    pub fn ground(&self) -> &dyn TerrainCostCalculator {
        self.get("ground").expect("GroundCostCalculator 必须存在")
    }

    /// 根据单位标签解析对应的成本计算器
    /// 优先级：SWIMMING > FLYING > MOUNTED > 默认(ground)
    pub fn resolve_from_tags(&self, tags: &GameplayTags) -> &dyn TerrainCostCalculator {
        if tags.has(GameplayTag::SWIMMING) {
            return self.get("swimming").unwrap_or(self.ground());
        }
        if tags.has(GameplayTag::FLYING) {
            return self.get("flying").unwrap_or(self.ground());
        }
        if tags.has(GameplayTag::MOUNTED) {
            return self.get("mounted").unwrap_or(self.ground());
        }
        self.ground()
    }
}

// ── 寻路核心 ──

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

#[cfg(test)]
mod tests {
    use super::*;

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
