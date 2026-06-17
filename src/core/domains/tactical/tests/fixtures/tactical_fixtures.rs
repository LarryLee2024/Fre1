use bevy::prelude::Entity;

use crate::core::domains::tactical::components::{GridPos, MovementPoints, MovementType};
use crate::core::domains::tactical::resources::{GridLayout, GridMap, TileData, TileFlags};

/// 创建一个 10x10 的默认网格。
pub fn default_grid() -> GridMap {
    GridMap::new(10, 10, GridLayout::Square)
}

/// 创建一个 5x5 的基础网格，部分 Tile 不可通行。
pub fn grid_with_obstacles() -> GridMap {
    let mut map = GridMap::new(5, 5, GridLayout::Square);
    // Block a wall at x=2
    for y in 0..5 {
        let pos = GridPos::new(2, y);
        if let Some(tile) = map.get_tile_mut(pos) {
            *tile = TileData::new(0, 0, TileFlags(0));
        }
    }
    map
}

/// 创建一个指定尺寸的网格，所有 Tile 通行。
pub fn grid_of_size(width: u32, height: u32) -> GridMap {
    GridMap::new(width, height, GridLayout::Square)
}

/// 创建一个拥有指定 MP 的实体组件数据。
pub fn walk_mp(max: f32) -> MovementPoints {
    MovementPoints::new(max, MovementType::Walk)
}

pub fn fly_mp(max: f32) -> MovementPoints {
    MovementPoints::new(max, MovementType::Fly)
}

/// 创建一个 Entity 占位符（仅用于测试签名，不创建实际 Entity）。
pub fn dummy_entity() -> Entity {
    Entity::PLACEHOLDER
}

/// 一个标准的测试场景：5x5 网格、单位在 (0,0) 有 5 MP。
pub struct TestGridScenario {
    pub grid: GridMap,
    pub unit_pos: GridPos,
    pub unit_mp: MovementPoints,
}

impl Default for TestGridScenario {
    fn default() -> Self {
        Self {
            grid: default_grid(),
            unit_pos: GridPos::new(0, 0),
            unit_mp: walk_mp(5.0),
        }
    }
}

impl TestGridScenario {
    pub fn with_obstacles() -> Self {
        Self {
            grid: grid_with_obstacles(),
            unit_pos: GridPos::new(0, 0),
            unit_mp: walk_mp(5.0),
        }
    }
}
