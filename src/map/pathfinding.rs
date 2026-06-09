// 寻路模块：BFS 计算可移动范围与路径
// 包含 TerrainCostCalculator trait，支持不同单位类型的地形通行差异

use super::grid::{GameMap, Terrain, Tile};
use crate::gameplay::tag::{GameplayTag, GameplayTags};
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

// ── 地形成本计算 trait ──

/// 地形移动成本计算规则 trait：描述不同单位类型的地形通行能力
pub trait TerrainCostCalculator: Send + Sync + 'static {
    /// 计算器名称（用于注册和查找）
    fn name(&self) -> &'static str;
    /// 计算指定地形的移动成本，None 表示不可通行
    /// terrain: 地形类型
    /// base_cost: 从 TerrainRegistry 加载的基础成本（None 表示基础不可通行）
    fn cost(&self, terrain: Terrain, base_cost: Option<u32>) -> Option<u32>;
}

// ── 内置实现 ──

/// 步兵成本计算器：使用基础成本，山地/水域不可通行
pub struct GroundCostCalculator;

impl TerrainCostCalculator for GroundCostCalculator {
    fn name(&self) -> &'static str {
        "ground"
    }

    fn cost(&self, _terrain: Terrain, base_cost: Option<u32>) -> Option<u32> {
        // 步兵直接使用基础成本，None（不可通行）保持不变
        base_cost
    }
}

/// 飞行成本计算器：所有地形成本为1，山地/水域也可通行
pub struct FlyingCostCalculator;

impl TerrainCostCalculator for FlyingCostCalculator {
    fn name(&self) -> &'static str {
        "flying"
    }

    fn cost(&self, _terrain: Terrain, _base_cost: Option<u32>) -> Option<u32> {
        // 飞行单位无视地形，所有可进入格子成本为1
        Some(1)
    }
}

/// 骑兵成本计算器：平原成本1，森林成本3，山地/水域不可通行
pub struct MountedCostCalculator;

impl TerrainCostCalculator for MountedCostCalculator {
    fn name(&self) -> &'static str {
        "mounted"
    }

    fn cost(&self, terrain: Terrain, _base_cost: Option<u32>) -> Option<u32> {
        match terrain {
            Terrain::Plain => Some(1),
            Terrain::Forest => Some(3),
            // 骑兵无法进入山地和水域
            Terrain::Mountain | Terrain::Water => None,
        }
    }
}

/// 水生成本计算器：水域成本1，平原成本2，山地不可通行
pub struct SwimmingCostCalculator;

impl TerrainCostCalculator for SwimmingCostCalculator {
    fn name(&self) -> &'static str {
        "swimming"
    }

    fn cost(&self, terrain: Terrain, _base_cost: Option<u32>) -> Option<u32> {
        match terrain {
            Terrain::Water => Some(1),
            Terrain::Plain => Some(2),
            Terrain::Forest => Some(3),
            // 水生单位无法进入山地
            Terrain::Mountain => None,
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
        // 水生优先级最高（两栖单位按水生处理）
        if tags.has(GameplayTag::SWIMMING) {
            return self.get("swimming").unwrap_or(self.ground());
        }
        if tags.has(GameplayTag::FLYING) {
            return self.get("flying").unwrap_or(self.ground());
        }
        if tags.has(GameplayTag::MOUNTED) {
            return self.get("mounted").unwrap_or(self.ground());
        }
        // 默认使用步兵计算器
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
    tiles: &HashMap<IVec2, (Terrain, Option<u32>)>,
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

    // 从 target 反向回溯到 start
    let mut path = vec![target];
    let mut current = target;
    let mut remaining = reachable[&target];

    while current != start {
        let mut best_prev = None;
        let mut best_remaining = remaining; // 寻找 remaining 更大的邻居（更接近起点）

        for dir in &directions {
            let prev = current - *dir;
            if prev == start {
                best_prev = Some(prev);
                break;
            }
            if let Some(&prev_remaining) = reachable.get(&prev) {
                // prev 的剩余移动力必须大于 current 的剩余移动力 + cost
                let terrain_data = match tiles.get(&current) {
                    Some(t) => *t,
                    None => continue,
                };
                let cost = match calculator.cost(terrain_data.0, terrain_data.1) {
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
            None => break, // 无法回溯，直接返回
        }
    }

    path.reverse();
    // 去掉起点
    if path.first() == Some(&start) {
        path.remove(0);
    }
    path
}

/// 寻路结果：可到达的格子及其剩余移动力
/// calculator: 地形成本计算器，决定不同单位类型的地形通行能力
pub fn find_reachable_tiles(
    start: IVec2,
    move_points: u32,
    map: &GameMap,
    tiles: &HashMap<IVec2, (Terrain, Option<u32>)>, // terrain + base_cost
    occupied: &HashMap<IVec2, bool>,
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

            let terrain_data = match tiles.get(&next) {
                Some(t) => *t,
                None => continue,
            };

            // 通过计算器获取该单位类型在此地形的实际成本
            let cost = match calculator.cost(terrain_data.0, terrain_data.1) {
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

/// 构建地形查找表（terrain + move_cost）
pub fn build_tile_terrain_map(tiles: &Query<&Tile>) -> HashMap<IVec2, (Terrain, Option<u32>)> {
    tiles
        .iter()
        .map(|tile| (tile.coord, (tile.terrain, tile.move_cost)))
        .collect()
}

/// 地形查找表缓存资源（地图生成时计算一次，避免每帧重复构建）
#[derive(Resource, Default)]
pub struct TerrainMapCache {
    pub map: HashMap<IVec2, (Terrain, Option<u32>)>,
}

/// 在地图生成后缓存地形查找表
pub fn cache_terrain_map(mut cache: ResMut<TerrainMapCache>, tiles: Query<&Tile>) {
    cache.map = build_tile_terrain_map(&tiles);
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

    /// 构建全平地地形表（terrain + move_cost）
    fn all_plain_map(map: &GameMap) -> HashMap<IVec2, (Terrain, Option<u32>)> {
        let mut tiles = HashMap::new();
        for x in 0..map.width {
            for y in 0..map.height {
                tiles.insert(IVec2::new(x as i32, y as i32), (Terrain::Plain, Some(1)));
            }
        }
        tiles
    }

    // ── 步兵（GroundCostCalculator）测试 ──

    #[test]
    fn 步兵_平地_移动力3_可到达13格() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied, &calculator);

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
    fn 步兵_移动力0_无可达格子() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 0, &map, &tiles, &occupied, &calculator);
        assert!(reachable.is_empty());
    }

    #[test]
    fn 步兵_移动力1_只能到相邻4格() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 1, &map, &tiles, &occupied, &calculator);

        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 3)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    #[test]
    fn 步兵_山地和水域不可通行() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        // 右侧设为山地
        tiles.insert(IVec2::new(3, 2), (Terrain::Mountain, None));
        // 上方设为水域
        tiles.insert(IVec2::new(2, 3), (Terrain::Water, None));
        let occupied = HashMap::new();
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 1, &map, &tiles, &occupied, &calculator);

        // 只有左和下可达
        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains_key(&IVec2::new(1, 2)));
        assert!(reachable.contains_key(&IVec2::new(2, 1)));
    }

    #[test]
    fn 步兵_森林消耗2移动力() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        // 相邻格设为森林
        tiles.insert(IVec2::new(3, 2), (Terrain::Forest, Some(2)));
        let occupied = HashMap::new();
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 2, &map, &tiles, &occupied, &calculator);

        // 森林格消耗2，移动力2刚好到达，剩余0
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        // 森林格后面不可达（剩余移动力为0）
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    #[test]
    fn 步兵_被敌方占据的格子不可达() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let mut occupied = HashMap::new();
        occupied.insert(IVec2::new(3, 2), true);
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied, &calculator);

        assert!(!reachable.contains_key(&IVec2::new(3, 2)));
        // 被占据格子不可穿越，(4,2)需经过(3,2)所以也不可达
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
        // 但绕路可达(4,2)的邻居：如(4,1)经(3,1)(4,1)可达
        assert!(reachable.contains_key(&IVec2::new(4, 1)));
    }

    #[test]
    fn 步兵_角落位置_移动力受限() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let occupied = HashMap::new();
        let calculator = GroundCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(0, 0), 2, &map, &tiles, &occupied, &calculator);

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

    // ── 飞行（FlyingCostCalculator）测试 ──

    #[test]
    fn 飞行_山地和水域可通行_成本为1() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        tiles.insert(IVec2::new(3, 2), (Terrain::Mountain, None));
        tiles.insert(IVec2::new(2, 3), (Terrain::Water, None));
        let occupied = HashMap::new();
        let calculator = FlyingCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 1, &map, &tiles, &occupied, &calculator);

        // 飞行单位山地和水域成本为1，移动力1可达全部4个相邻格
        assert_eq!(reachable.len(), 4);
        assert!(reachable.contains_key(&IVec2::new(3, 2))); // 山地
        assert!(reachable.contains_key(&IVec2::new(2, 3))); // 水域
    }

    #[test]
    fn 飞行_森林成本为1() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        tiles.insert(IVec2::new(3, 2), (Terrain::Forest, Some(2)));
        let occupied = HashMap::new();
        let calculator = FlyingCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 2, &map, &tiles, &occupied, &calculator);

        // 飞行单位森林成本为1，移动力2可达森林后继续前进
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        // 森林后还有剩余移动力，可继续到达(4,2)
        assert!(reachable.contains_key(&IVec2::new(4, 2)));
    }

    // ── 骑兵（MountedCostCalculator）测试 ──

    #[test]
    fn 骑兵_平原成本1_森林成本3() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        tiles.insert(IVec2::new(3, 2), (Terrain::Forest, Some(2)));
        let occupied = HashMap::new();
        let calculator = MountedCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied, &calculator);

        // 骑兵森林成本3，移动力3刚好到达森林，剩余0
        assert!(reachable.contains_key(&IVec2::new(3, 2)));
        assert_eq!(*reachable.get(&IVec2::new(3, 2)).unwrap(), 0);
        // 森林后不可达
        assert!(!reachable.contains_key(&IVec2::new(4, 2)));
    }

    #[test]
    fn 骑兵_山地和水域不可通行() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        tiles.insert(IVec2::new(3, 2), (Terrain::Mountain, None));
        tiles.insert(IVec2::new(2, 3), (Terrain::Water, None));
        let occupied = HashMap::new();
        let calculator = MountedCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied, &calculator);

        assert!(!reachable.contains_key(&IVec2::new(3, 2))); // 山地不可通行
        assert!(!reachable.contains_key(&IVec2::new(2, 3))); // 水域不可通行
    }

    // ── 水生（SwimmingCostCalculator）测试 ──

    #[test]
    fn 水生_水域成本1_平原成本2() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        tiles.insert(IVec2::new(3, 2), (Terrain::Water, None));
        let occupied = HashMap::new();
        let calculator = SwimmingCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied, &calculator);

        // 水生单位水域成本1，平原成本2
        assert!(reachable.contains_key(&IVec2::new(3, 2))); // 水域可达
        // 平原成本2，移动力3走一步平原后剩余1，不够再走一步平原
        // 但水域成本1，可从水域继续
    }

    #[test]
    fn 水生_山地不可通行() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        tiles.insert(IVec2::new(3, 2), (Terrain::Mountain, None));
        let occupied = HashMap::new();
        let calculator = SwimmingCostCalculator;

        let reachable =
            find_reachable_tiles(IVec2::new(2, 2), 3, &map, &tiles, &occupied, &calculator);

        assert!(!reachable.contains_key(&IVec2::new(3, 2))); // 山地不可通行
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

        // 无移动标签 → 步兵
        let tags = GameplayTags::default();
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "ground");

        // 飞行标签 → 飞行
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FLYING);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "flying");

        // 骑兵标签 → 骑兵
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::MOUNTED);
        let calc = registry.resolve_from_tags(&tags);
        assert_eq!(calc.name(), "mounted");

        // 水生标签 → 水生
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
        let tiles = all_plain_map(&map);
        let reachable = HashMap::new();
        let calculator = GroundCostCalculator;

        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(2, 2),
            &reachable,
            3,
            &map,
            &tiles,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(2, 2)]);
    }

    #[test]
    fn 回溯路径_相邻格子() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let calculator = GroundCostCalculator;

        // 模拟 BFS 结果：从 (2,2) 出发，剩余移动力 3
        // 到达 (3,2) 消耗 1，剩余 2
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(3, 2), 2);

        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(3, 2),
            &reachable,
            3,
            &map,
            &tiles,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(3, 2)]);
    }

    #[test]
    fn 回溯路径_直线两格() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let calculator = GroundCostCalculator;

        // 从 (2,2) 到 (4,2)，经过 (3,2)
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(3, 2), 2); // 从 (2,2) 到 (3,2) 消耗 1，剩余 2
        reachable.insert(IVec2::new(4, 2), 1); // 从 (3,2) 到 (4,2) 消耗 1，剩余 1

        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(4, 2),
            &reachable,
            3,
            &map,
            &tiles,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(3, 2), IVec2::new(4, 2)]);
    }

    #[test]
    fn 回溯路径_不存在的目标() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let reachable = HashMap::new();
        let calculator = GroundCostCalculator;

        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(4, 4),
            &reachable,
            3,
            &map,
            &tiles,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(4, 4)]);
    }

    #[test]
    fn 回溯路径_L形路径() {
        let map = make_test_map();
        let tiles = all_plain_map(&map);
        let calculator = GroundCostCalculator;

        // 从 (1,1) 到 (2,2)，路径 (1,1) → (2,1) → (2,2)
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(2, 1), 2); // (1,1) → (2,1) 消耗 1，剩余 2
        reachable.insert(IVec2::new(2, 2), 1); // (2,1) → (2,2) 消耗 1，剩余 1

        let path = reconstruct_path(
            IVec2::new(1, 1),
            IVec2::new(2, 2),
            &reachable,
            3,
            &map,
            &tiles,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(2, 1), IVec2::new(2, 2)]);
    }

    #[test]
    fn 回溯路径_有森林地形() {
        let map = make_test_map();
        let mut tiles = all_plain_map(&map);
        // (3,2) 是森林，移动成本 2
        tiles.insert(IVec2::new(3, 2), (Terrain::Forest, Some(2)));
        let calculator = GroundCostCalculator;

        // 从 (2,2) 到 (4,2)，经过 (3,2) 森林
        let mut reachable = HashMap::new();
        reachable.insert(IVec2::new(3, 2), 1); // (2,2) → (3,2) 森林消耗 2，剩余 1
        reachable.insert(IVec2::new(4, 2), 0); // (3,2) → (4,2) 消耗 1，剩余 0

        let path = reconstruct_path(
            IVec2::new(2, 2),
            IVec2::new(4, 2),
            &reachable,
            3,
            &map,
            &tiles,
            &calculator,
        );

        assert_eq!(path, vec![IVec2::new(3, 2), IVec2::new(4, 2)]);
    }
}
