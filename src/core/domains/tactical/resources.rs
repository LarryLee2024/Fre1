//! Resources — 网格数据（ECS Resource）
//!
//! GridMap 作为全局 Resource 存储所有 Tile 数据。
//! Tile 默认不实例化为 Entity（性能考虑），仅当需要时通过 TileMarker 实体化。
//!
//! 详见 ADR-022 §1

use bevy::prelude::*;

use super::components::GridPos;

/// 网格布局类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridLayout {
    /// 四向网格（简单）
    Square,
    /// 六边形，奇数列偏移
    HexRowOdd,
    /// 六边形，偶数列偏移
    HexRowEven,
    /// 六边形，奇数行偏移
    HexColOdd,
    /// 六边形，偶数列偏移
    HexColEven,
}

/// 单 Tile 的通行标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileFlags(pub u8);

impl TileFlags {
    pub const PASSABLE: Self = Self(0b0000_0001);
    pub const FLYABLE: Self = Self(0b0000_0010);
    pub const BUILDABLE: Self = Self(0b0000_0100);
    pub const BLOCKS_SIGHT: Self = Self(0b0000_1000);

    /// 检查是否包含指定标记位（位运算子集检查）。
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// 检查格子是否可通行（包含 PASSABLE 标记）。
    pub fn is_passable(&self) -> bool {
        self.contains(Self::PASSABLE)
    }
}

/// 单 Tile 数据 — 紧凑存储。
///
/// 使用 u32 打包：地形 ID(16bit) + 高度(8bit) + 标记(8bit)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileData {
    packed: u32,
}

impl TileData {
    const TERRAIN_MASK: u32 = 0xFFFF_0000;
    const HEIGHT_MASK: u32 = 0x0000_FF00;
    const FLAGS_MASK: u32 = 0x0000_00FF;
    const TERRAIN_SHIFT: u32 = 16;
    const HEIGHT_SHIFT: u32 = 8;

    /// 从地形 ID、高度和标记位创建紧凑 TileData。
    pub fn new(terrain_def_id: u16, height: u8, flags: TileFlags) -> Self {
        Self {
            packed: (terrain_def_id as u32) << Self::TERRAIN_SHIFT
                | (height as u32) << Self::HEIGHT_SHIFT
                | flags.0 as u32,
        }
    }

    /// 提取地形定义 ID（高 16 位）。
    pub fn terrain_def_id(&self) -> u16 {
        ((self.packed & Self::TERRAIN_MASK) >> Self::TERRAIN_SHIFT) as u16
    }

    /// 提取高度值（中间 8 位）。
    pub fn height(&self) -> u8 {
        ((self.packed & Self::HEIGHT_MASK) >> Self::HEIGHT_SHIFT) as u8
    }

    /// 提取标记位（低 8 位）。
    pub fn flags(&self) -> TileFlags {
        TileFlags((self.packed & Self::FLAGS_MASK) as u8)
    }

    /// 检查格子是否可通行（委托给 TileFlags::is_passable）。
    pub fn is_passable(&self) -> bool {
        self.flags().is_passable()
    }
}

/// 全局网格数据（Resource）。
///
/// 存储为 Resource 而非 Entity 集合 —— 网格 Tile 数量可能很大，
/// 每个 Tile 一个 Entity 会导致 Archetype 爆炸。
///
/// 🟥 禁止在 GridMap 中存储业务逻辑（单位列表、战斗状态等）。
#[derive(Resource, Debug, Clone)]
pub struct GridMap {
    /// 网格宽度
    pub width: u32,
    /// 网格高度
    pub height: u32,
    /// 平铺 Tile 数据（按行序排列：y * width + x）
    pub tiles: Vec<TileData>,
    /// 网格布局
    pub layout: GridLayout,
}

impl GridMap {
    /// 创建指定尺寸的网格，所有 Tile 初始化为默认值。
    pub fn new(width: u32, height: u32, layout: GridLayout) -> Self {
        let tile_count = (width * height) as usize;
        Self {
            width,
            height,
            tiles: vec![TileData::new(0, 0, TileFlags::PASSABLE); tile_count],
            layout,
        }
    }

    /// 从现有数据创建网格。
    pub fn from_tiles(width: u32, height: u32, tiles: Vec<TileData>, layout: GridLayout) -> Self {
        assert_eq!(
            tiles.len(),
            (width * height) as usize,
            "tile count must match width * height"
        );
        Self {
            width,
            height,
            tiles,
            layout,
        }
    }

    /// 检查坐标是否在网格范围内。
    pub fn in_bounds(&self, pos: GridPos) -> bool {
        pos.x >= 0 && (pos.x as u32) < self.width && pos.y >= 0 && (pos.y as u32) < self.height
    }

    /// 将 (x, y) 转换为平铺索引。
    fn index(&self, pos: GridPos) -> Option<usize> {
        if self.in_bounds(pos) {
            Some((pos.y as usize) * (self.width as usize) + (pos.x as usize))
        } else {
            None
        }
    }

    /// 获取 Tile 数据（只读）。
    pub fn get_tile(&self, pos: GridPos) -> Option<&TileData> {
        self.index(pos).map(|i| &self.tiles[i])
    }

    /// 获取 Tile 数据（可变）。
    pub fn get_tile_mut(&mut self, pos: GridPos) -> Option<&mut TileData> {
        self.index(pos).map(move |i| &mut self.tiles[i])
    }

    /// 四连通邻居。
    pub fn neighbors_4(&self, pos: GridPos) -> Vec<GridPos> {
        pos.neighbors_4()
            .into_iter()
            .filter(|&p| self.in_bounds(p))
            .collect()
    }

    /// 范围内的所有可达 Tile（BFS 漫水）。
    ///
    /// 从 center 开始，在 range 步内找到所有可通行的格子。
    pub fn tiles_in_range(&self, center: GridPos, range: u32) -> Vec<GridPos> {
        if !self.in_bounds(center) {
            return Vec::new();
        }

        let mut visited = vec![false; self.tiles.len()];
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        if let Some(idx) = self.index(center) {
            visited[idx] = true;
        }
        queue.push_back((center, 0));

        while let Some((pos, dist)) = queue.pop_front() {
            if dist >= range {
                continue;
            }

            for neighbor in self.neighbors_4(pos) {
                if let Some(idx) = self.index(neighbor)
                    && !visited[idx]
                {
                    visited[idx] = true;
                    let tile = &self.tiles[idx];
                    if tile.is_passable() {
                        result.push(neighbor);
                        queue.push_back((neighbor, dist + 1));
                    }
                }
            }
        }

        result
    }

    /// 网格坐标 → 世界坐标（简化版，假设 Tile 大小为 1.0）。
    pub fn grid_to_world(&self, pos: GridPos) -> (f32, f32) {
        match self.layout {
            GridLayout::Square => (pos.x as f32, pos.y as f32),
            GridLayout::HexRowOdd | GridLayout::HexRowEven => {
                let x = pos.x as f32 * 1.5;
                let y = pos.y as f32 * 1.732 + if pos.x % 2 == 1 { 0.866 } else { 0.0 };
                (x, y)
            }
            GridLayout::HexColOdd | GridLayout::HexColEven => {
                let x = pos.x as f32 * 1.732 + if pos.y % 2 == 1 { 0.866 } else { 0.0 };
                let y = pos.y as f32 * 1.5;
                (x, y)
            }
        }
    }

    /// 世界坐标 → 网格坐标（简化版）。
    pub fn world_to_grid(&self, wx: f32, wy: f32) -> Option<GridPos> {
        match self.layout {
            GridLayout::Square => {
                let x = wx.round() as i32;
                let y = wy.round() as i32;
                let pos = GridPos::new(x, y);
                if self.in_bounds(pos) { Some(pos) } else { None }
            }
            _ => {
                // Hex 转换简化：先做方形近似
                let x = (wx / 1.5).round() as i32;
                let y = (wy / 1.732).round() as i32;
                let pos = GridPos::new(x, y);
                if self.in_bounds(pos) { Some(pos) } else { None }
            }
        }
    }
}
