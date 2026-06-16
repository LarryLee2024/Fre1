// 地形网格：纯数据存储地形 ID，替代 Tile Entity
// TerrainGrid 是地形数据的唯一真相源，寻路/UI/战斗都从这里读取

use bevy::prelude::*;
use std::collections::HashMap;

/// 地形网格资源：存储每个坐标的地形 ID
/// 替代原来的 Tile Entity + Terrain 枚举
/// 地形 ID（如 "plain", "forest"）由 TerrainRegistry 定义
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct TerrainGrid {
    /// 地图宽度
    pub width: u32,
    /// 地图高度
    pub height: u32,
    /// (x, y) → terrain_id
    cells: HashMap<IVec2, String>,
}

impl TerrainGrid {
    /// 从 LevelConfig 的 terrain_map 构建
    pub fn from_terrain_map(
        width: u32,
        height: u32,
        terrain_map: &HashMap<(i32, i32), String>,
    ) -> Self {
        let mut cells = HashMap::new();
        for y in 0..height {
            for x in 0..width {
                let coord = IVec2::new(x as i32, y as i32);
                let terrain_id = terrain_map
                    .get(&(x as i32, y as i32))
                    .cloned()
                    .unwrap_or_else(|| "plain".to_string());
                cells.insert(coord, terrain_id);
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }

    /// 获取指定坐标的地形 ID
    pub fn get(&self, coord: IVec2) -> Option<&str> {
        self.cells.get(&coord).map(|s| s.as_str())
    }

    /// 设置指定坐标的地形 ID
    pub fn set(&mut self, coord: IVec2, terrain_id: String) {
        self.cells.insert(coord, terrain_id);
    }

    /// 检查坐标是否在网格范围内
    pub fn is_in_bounds(&self, coord: IVec2) -> bool {
        coord.x >= 0
            && coord.y >= 0
            && (coord.x as u32) < self.width
            && (coord.y as u32) < self.height
    }

    /// 迭代所有格子
    pub fn iter(&self) -> impl Iterator<Item = (IVec2, &str)> {
        self.cells.iter().map(|(coord, id)| (*coord, id.as_str()))
    }

    /// 兜底默认地形网格（全平地）
    pub fn default_plain(width: u32, height: u32) -> Self {
        let mut cells = HashMap::new();
        for y in 0..height {
            for x in 0..width {
                cells.insert(IVec2::new(x as i32, y as i32), "plain".to_string());
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }
}

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

    /// Test ID: MAP-TGR-001
    /// Title: 从地形 map 构建 TerrainGrid
    ///
    /// Given: 包含 plain/forest/mountain 的地形映射
    /// When: 调用 TerrainGrid::from_terrain_map()
    /// Then: 正确构建地形网格，未配置格子默认 plain
    ///
    /// Assertions: get() 返回正确地形 ID
    #[test]
    fn 从地形map构建() {
        // Given
        let mut terrain_map = HashMap::new();
        terrain_map.insert((0, 0), "plain".to_string());
        terrain_map.insert((1, 0), "forest".to_string());
        terrain_map.insert((0, 1), "mountain".to_string());

        // When
        let grid = TerrainGrid::from_terrain_map(2, 2, &terrain_map);

        // Then
        assert_eq!(grid.get(IVec2::new(0, 0)), Some("plain"));
        assert_eq!(grid.get(IVec2::new(1, 0)), Some("forest"));
        assert_eq!(grid.get(IVec2::new(0, 1)), Some("mountain"));
        assert_eq!(grid.get(IVec2::new(1, 1)), Some("plain"));
    }

    /// Test ID: MAP-TGR-002
    /// Title: TerrainGrid 边界检查
    ///
    /// Given: 3x3 TerrainGrid
    /// When: 检查不同坐标的边界
    /// Then: 正确判断坐标是否在范围内
    ///
    /// Assertions: is_in_bounds() 返回正确的 bool
    #[test]
    fn 边界检查() {
        // Given
        let grid = TerrainGrid::default_plain(3, 3);

        // When & Then
        assert!(grid.is_in_bounds(IVec2::new(0, 0)));
        assert!(grid.is_in_bounds(IVec2::new(2, 2)));
        assert!(!grid.is_in_bounds(IVec2::new(3, 0)));
        assert!(!grid.is_in_bounds(IVec2::new(-1, 0)));
    }

    /// Test ID: MAP-TGR-003
    /// Title: 设置地形
    ///
    /// Given: 3x3 全平地 TerrainGrid
    /// When: 设置 (1,1) 为 water
    /// Then: get(1,1) 返回 "water"
    ///
    /// Assertions: get() 返回 "water"
    #[test]
    fn 设置地形() {
        // Given
        let mut grid = TerrainGrid::default_plain(3, 3);

        // When
        grid.set(IVec2::new(1, 1), "water".to_string());

        // Then
        assert_eq!(grid.get(IVec2::new(1, 1)), Some("water"));
    }

    /// Test ID: MAP-TGR-004
    /// Title: 迭代所有格子
    ///
    /// Given: 2x2 TerrainGrid
    /// When: 调用 iter().count()
    /// Then: 返回 4 个格子
    ///
    /// Assertions: count == 4
    #[test]
    fn 迭代所有格子() {
        // Given
        let grid = TerrainGrid::default_plain(2, 2);

        // When
        let count = grid.iter().count();

        // Then
        assert_eq!(count, 4);
    }
}
