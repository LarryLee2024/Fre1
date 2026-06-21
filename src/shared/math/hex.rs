//! 六边形网格坐标工具
//!
//! 使用轴向坐标 (q, r) 表示六边形网格中的位置。
//! 第三个立方体坐标 `s = -q - r`，用于距离计算。
//!
//! # 坐标系
//!
//! 采用点顶朝向（pointy-top）轴向坐标系：
//!
//! ```text
//!     _____
//!    /     \
//!   /       \
//!  /    q    \
//! /____     __\
//! \     \  /   /
//!  \  r  \/   /
//!   \     \  /
//!    \_____\/
//! ```
//!
//! 六个邻居方向：
//! - (q+1, r),   (q-1, r)
//! - (q,   r+1), (q,   r-1)
//! - (q+1, r-1), (q-1, r+1)

use std::ops::{Add, Sub};

/// 六边形轴向坐标。
///
/// 使用 (q, r) 表示，满足 `s = -q - r` 关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    /// 列坐标
    pub q: i32,
    /// 行坐标
    pub r: i32,
}

impl HexCoord {
    /// 从轴向坐标 (q, r) 创建新的六边形坐标。
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// 计算到另一个六边形坐标的立方体距离。
    ///
    /// 使用立方体坐标距离公式 `(|dq| + |dr| + |ds|) / 2`。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let a = HexCoord::new(0, 0);
    /// let b = HexCoord::new(2, 1);
    /// assert_eq!(a.distance_to(&b), 3);
    /// ```
    pub fn distance_to(&self, other: &HexCoord) -> i32 {
        hex_distance((self.q, self.r), (other.q, other.r))
    }

    /// 返回六个邻居的坐标数组。
    ///
    /// 顺序为：右上、右、右下、左下、左、左上。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let center = HexCoord::new(0, 0);
    /// let neighbors = center.neighbors();
    /// assert_eq!(neighbors.len(), 6);
    /// assert!(neighbors.contains(&HexCoord::new(1, 0)));
    /// ```
    pub fn neighbors(&self) -> [HexCoord; 6] {
        let (q, r) = (self.q, self.r);
        [
            HexCoord::new(q + 1, r),     // 右上
            HexCoord::new(q - 1, r),     // 左下
            HexCoord::new(q, r + 1),     // 右下
            HexCoord::new(q, r - 1),     // 左上
            HexCoord::new(q + 1, r - 1), // 右（对点顶朝上反而是左上）
            HexCoord::new(q - 1, r + 1), // 左（对点顶朝上反而是右下）
        ]
    }
}

impl Add for HexCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl Sub for HexCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}

impl From<(i32, i32)> for HexCoord {
    fn from((q, r): (i32, i32)) -> Self {
        Self { q, r }
    }
}

/// 计算两个六边形轴向坐标之间的立方体距离。
///
/// 距离定义为立方体坐标曼哈顿距离的一半：
///
/// ```text
/// distance(a, b) = (|dq| + |dr| + |dq + dr|) / 2
/// ```
///
/// 其中 `dq = b.q - a.q`，`dr = b.r - a.r`。
///
/// # 示例
///
/// ```ignore
/// assert_eq!(hex_distance((0, 0), (2, 1)), 3);
/// assert_eq!(hex_distance((0, 0), (0, 0)), 0);
/// assert_eq!(hex_distance((-3, 2), (1, -1)), 5);
/// ```
pub fn hex_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    let dq = b.0 - a.0;
    let dr = b.1 - a.1;
    // s = -q - r, so ds = -dq - dr
    // distance = (|dq| + |dr| + |ds|) / 2 = (|dq| + |dr| + |dq + dr|) / 2
    (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
}
