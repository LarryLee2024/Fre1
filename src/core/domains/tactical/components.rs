//! ECS Components — 战术空间组件
//!
//! 每个战场单位的基础空间数据：位置、移动力、朝向。
//! 详见 docs/02-domain/domains/tactical_domain.md

use bevy::prelude::*;

/// 网格坐标 — 单位在战场上的位置。
///
/// 使用立方体坐标的变体，支持 Square 和 Hex 两种布局。
/// x/y 为轴向坐标，layer 为层高。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
    pub layer: i8,
}

impl GridPos {
    /// 创建新坐标。
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y, layer: 0 }
    }

    /// 创建带层高的坐标。
    pub const fn with_layer(x: i32, y: i32, layer: i8) -> Self {
        Self { x, y, layer }
    }

    /// 曼哈顿距离（Square 网格）。
    pub fn manhattan_distance(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }

    /// 切比雪夫距离（Square 网格，允许对角线移动）。
    pub fn chebyshev_distance(self, other: Self) -> u32 {
        self.x.abs_diff(other.x).max(self.y.abs_diff(other.y))
    }

    /// 六边形距离（轴向坐标）。
    pub fn hex_distance(self, other: Self) -> u32 {
        let dx = (self.x - other.x).unsigned_abs();
        let dy = (self.y - other.y).unsigned_abs();
        // 六边形轴向坐标距离: (|dx| + |dy| + |dx - dy|) / 2
        (dx + dy + (self.x - other.x).abs_diff(self.y - other.y)) / 2
    }

    /// 四连通邻居（上/下/左/右）。
    pub fn neighbors_4(&self) -> [Self; 4] {
        [
            Self::with_layer(self.x, self.y - 1, self.layer),
            Self::with_layer(self.x, self.y + 1, self.layer),
            Self::with_layer(self.x - 1, self.y, self.layer),
            Self::with_layer(self.x + 1, self.y, self.layer),
        ]
    }

    /// 八连通邻居（含对角线）。
    pub fn neighbors_8(&self) -> [Self; 8] {
        [
            Self::with_layer(self.x - 1, self.y - 1, self.layer),
            Self::with_layer(self.x, self.y - 1, self.layer),
            Self::with_layer(self.x + 1, self.y - 1, self.layer),
            Self::with_layer(self.x - 1, self.y, self.layer),
            Self::with_layer(self.x + 1, self.y, self.layer),
            Self::with_layer(self.x - 1, self.y + 1, self.layer),
            Self::with_layer(self.x, self.y + 1, self.layer),
            Self::with_layer(self.x + 1, self.y + 1, self.layer),
        ]
    }
}

/// 移动力 — 单位在当前回合的移动能力。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct MovementPoints {
    /// 当前剩余移动力
    pub current: f32,
    /// 最大移动力
    pub max: f32,
    /// 已消耗的移动力
    pub consumed: f32,
    /// 移动类型
    pub movement_type: MovementType,
}

impl MovementPoints {
    /// 创建新的移动力组件。
    pub fn new(max: f32, movement_type: MovementType) -> Self {
        Self {
            current: max,
            max,
            consumed: 0.0,
            movement_type,
        }
    }

    /// 消耗移动力。返回 true 如果足够且消耗成功。
    pub fn consume(&mut self, cost: f32) -> bool {
        if cost > self.current {
            return false;
        }
        self.current -= cost;
        self.consumed += cost;
        true
    }

    /// 重置为最大值（回合开始调用）。
    pub fn reset(&mut self) {
        self.current = self.max;
        self.consumed = 0.0;
    }
}

/// 移动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum MovementType {
    Walk,
    Fly,
    Swim,
    Climb,
    Teleport,
}

/// 单位朝向（六边形方向）。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct Facing {
    pub direction: HexDirection,
}

impl Facing {
    pub fn new(direction: HexDirection) -> Self {
        Self { direction }
    }
}

/// 六边形方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum HexDirection {
    N,
    NE,
    SE,
    S,
    SW,
    NW,
}

impl HexDirection {
    /// 所有方向的数组。
    pub const ALL: [Self; 6] = [Self::N, Self::NE, Self::SE, Self::S, Self::SW, Self::NW];

    /// 方向向量（轴向坐标）。
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Self::N => (0, -1),
            Self::NE => (1, -1),
            Self::SE => (1, 0),
            Self::S => (0, 1),
            Self::SW => (-1, 1),
            Self::NW => (-1, 0),
        }
    }
}
