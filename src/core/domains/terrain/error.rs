//! 领域错误 — Terrain 域错误枚举。
//!
//! 涵盖地形通行性、表面变化、陷阱触发等操作的错误。
//! 详见 docs/02-domain/domains/terrain_domain.md §4

use bevy::prelude::*;

/// 地形系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum TerrainError {
    /// 格子坐标超出地图范围。
    OutOfBounds { x: i32, y: i32 },
    /// 格子不可通行。
    TileNotPassable { x: i32, y: i32 },
    /// 互斥表面类型冲突（如冰面和灼烧不可共存）。
    ConflictingSurfaceType,
    /// 相邻格高度差超过允许的最大值。
    HeightDifferenceExceeded { max_allowed: i32, actual: i32 },
    /// 陷阱缺少必要的触发条件或效果定义。
    InvalidHazardDefinition,
    /// 格子 ID 未注册。
    TileNotFound { x: i32, y: i32 },
}

impl std::fmt::Display for TerrainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds { x, y } => {
                write!(f, "tile coordinates out of bounds: ({}, {})", x, y)
            }
            Self::TileNotPassable { x, y } => {
                write!(f, "tile is not passable at ({}, {})", x, y)
            }
            Self::ConflictingSurfaceType => {
                write!(f, "conflicting surface types cannot coexist on same tile")
            }
            Self::HeightDifferenceExceeded {
                max_allowed,
                actual,
            } => {
                write!(
                    f,
                    "height difference between adjacent tiles exceeded: max={}, actual={}",
                    max_allowed, actual
                )
            }
            Self::InvalidHazardDefinition => {
                write!(
                    f,
                    "hazard zone missing required trigger or effect definition"
                )
            }
            Self::TileNotFound { x, y } => {
                write!(f, "tile not found at ({}, {})", x, y)
            }
        }
    }
}

impl std::error::Error for TerrainError {}
