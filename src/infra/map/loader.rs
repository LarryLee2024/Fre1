//! Loader — 运行时 MapAsset → ECS 状态转换
//!
//! 将 MapAsset 转换为 GridMap Resource 和相关的 ECS 组件。
//! 运行时关键路径，必须保持确定性和可重放性。
//!
//! 核心转换：
//! - TileEntry.terrain_id (String) → u16 索引（查 TerrainIndex）
//! - MapTileFlags (MapAsset 层) → TileFlags (Tactical domain)
//! - MapGridLayout → GridLayout 枚举映射
//!
//! 详见 ADR-065 §5 (Runtime 加载流程)

use std::collections::HashMap;

use bevy::prelude::Resource;

use crate::core::domains::tactical::components::HexDirection;
use crate::core::domains::tactical::resources::{GridLayout, GridMap, TileData, TileFlags};

use super::asset::MapAsset;
use super::types::{MapGridLayout, MapHexDirection, MapTileFlags};

// ─── TerrainIndex ───────────────────────────────────────────────

/// 地形索引 — terrain_id (String) → u16 索引的运行时映射。
///
/// 由 LoadedTerrainDefs 在加载时构建，供 MapAsset 转换使用。
/// u16 索引在 TileData 中作为高 16 位存储。
#[derive(Resource, Debug, Clone, Default)]
pub struct TerrainIndex {
    /// terrain_id → u16 索引的映射表。
    pub mapping: HashMap<String, u16>,
}

impl TerrainIndex {
    /// 查询 terrain_id 对应的 u16 索引。
    ///
    /// 未找到时返回 0（默认/未知地形）。
    pub fn get(&self, terrain_id: &str) -> u16 {
        self.mapping.get(terrain_id).copied().unwrap_or(0)
    }
}

/// 从 TerrainDef 列表构建 TerrainIndex。
///
/// terrain_defs 的迭代顺序决定 u16 索引值。
/// 索引 0 保留给默认/未找到的地形。
pub fn build_terrain_index(defs: &[crate::content::terrain_def::TerrainDef]) -> TerrainIndex {
    let mut mapping = HashMap::new();
    for (i, def) in defs.iter().enumerate() {
        mapping.insert(def.id.clone(), (i + 1) as u16);
    }
    TerrainIndex { mapping }
}

// ─── MapAsset → GridMap ─────────────────────────────────────────

/// 将整个 MapAsset 转换为 GridMap。
///
/// 转换过程：
/// 1. 转换网格布局类型
/// 2. 遍历每个 TileEntry：查询 TerrainIndex 获取 u16，转换 flags
/// 3. 构造 GridMap
pub fn convert_to_gridmap(map: &MapAsset, index: &TerrainIndex) -> GridMap {
    let width = map.metadata.width;
    let height = map.metadata.height;
    let layout = convert_grid_layout(map.metadata.layout);
    let mut tiles = Vec::with_capacity((width * height) as usize);

    for tile in &map.terrain_grid.tiles {
        let terrain_id = index.get(&tile.terrain_id);
        let flags = convert_tile_flags(tile.flags);
        tiles.push(TileData::new(terrain_id, tile.height, flags));
    }

    GridMap::from_tiles(width, height, tiles, layout)
}

// ─── 类型转换函数 ────────────────────────────────────────────────

/// 转换 MapGridLayout (infra/map) → GridLayout (tactical domain)。
pub fn convert_grid_layout(layout: MapGridLayout) -> GridLayout {
    match layout {
        MapGridLayout::Square => GridLayout::Square,
        MapGridLayout::HexRowOdd => GridLayout::HexRowOdd,
        MapGridLayout::HexRowEven => GridLayout::HexRowEven,
        MapGridLayout::HexColOdd => GridLayout::HexColOdd,
        MapGridLayout::HexColEven => GridLayout::HexColEven,
    }
}

/// 转换 MapTileFlags (infra/map) → TileFlags (tactical domain)。
///
/// 两个类型具有相同的 bit 布局（PASSABLE=0x01, FLYABLE=0x02 等），
/// 直接复用底层的 u8 值。
pub fn convert_tile_flags(flags: MapTileFlags) -> TileFlags {
    TileFlags(flags.0)
}

/// 转换 MapHexDirection (infra/map) → HexDirection (tactical domain)。
pub fn convert_hex_direction(dir: MapHexDirection) -> HexDirection {
    match dir {
        MapHexDirection::North => HexDirection::N,
        MapHexDirection::NorthEast => HexDirection::NE,
        MapHexDirection::SouthEast => HexDirection::SE,
        MapHexDirection::South => HexDirection::S,
        MapHexDirection::SouthWest => HexDirection::SW,
        MapHexDirection::NorthWest => HexDirection::NW,
        // 方形网格下的 Extra Direction 映射到最接近的 Hex 方向
        MapHexDirection::East => HexDirection::NE,
        MapHexDirection::West => HexDirection::NW,
    }
}

// ─── Renderer Helper: 从 GridMap → 屏幕坐标 ────────────────────

/// 计算指定 Tile 的世界坐标（用于 Entity-per-Tile 渲染）。
///
/// 委托给 GridMap::grid_to_world，乘以 TILE_SIZE 常量得到像素坐标。
pub const TILE_SIZE: f32 = 64.0;

/// 获取 Tile 在世界空间中的位置。
pub fn tile_world_position(grid_map: &GridMap, x: u32, y: u32) -> (f32, f32) {
    use crate::core::domains::tactical::components::GridPos;
    let pos = GridPos::new(x as i32, y as i32);
    let (wx, wy) = grid_map.grid_to_world(pos);
    (wx * TILE_SIZE, wy * TILE_SIZE)
}
