// 地形网格：纯数据存储地形 ID，替代 Tile Entity
// TerrainGrid 是地形数据的唯一真相源，寻路/UI/战斗都从这里读取

use bevy::prelude::*;
use std::collections::HashMap;

/// 地形网格资源：存储每个坐标的地形 ID
/// 替代原来的 Tile Entity + Terrain 枚举
/// 地形 ID（如 "plain", "forest"）由 TerrainRegistry 定义
#[derive(Resource, Debug, Clone)]
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
    use super::*;

    #[test]
    fn 从地形map构建() {
        let mut terrain_map = HashMap::new();
        terrain_map.insert((0, 0), "plain".to_string());
        terrain_map.insert((1, 0), "forest".to_string());
        terrain_map.insert((0, 1), "mountain".to_string());

        let grid = TerrainGrid::from_terrain_map(2, 2, &terrain_map);
        assert_eq!(grid.get(IVec2::new(0, 0)), Some("plain"));
        assert_eq!(grid.get(IVec2::new(1, 0)), Some("forest"));
        assert_eq!(grid.get(IVec2::new(0, 1)), Some("mountain"));
        // 未配置的格子默认为 plain
        assert_eq!(grid.get(IVec2::new(1, 1)), Some("plain"));
    }

    #[test]
    fn 边界检查() {
        let grid = TerrainGrid::default_plain(3, 3);
        assert!(grid.is_in_bounds(IVec2::new(0, 0)));
        assert!(grid.is_in_bounds(IVec2::new(2, 2)));
        assert!(!grid.is_in_bounds(IVec2::new(3, 0)));
        assert!(!grid.is_in_bounds(IVec2::new(-1, 0)));
    }

    #[test]
    fn 设置地形() {
        let mut grid = TerrainGrid::default_plain(3, 3);
        grid.set(IVec2::new(1, 1), "water".to_string());
        assert_eq!(grid.get(IVec2::new(1, 1)), Some("water"));
    }

    #[test]
    fn 迭代所有格子() {
        let grid = TerrainGrid::default_plain(2, 2);
        let count = grid.iter().count();
        assert_eq!(count, 4);
    }
}
