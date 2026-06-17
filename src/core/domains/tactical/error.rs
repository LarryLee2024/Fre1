//! 领域错误 — Tactical 域专属错误枚举。
//!
//! 遵循宪法 §8：分领域错误枚举，禁止全局 AppError。

use bevy::prelude::*;

/// Tactical 领域错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum TacticalError {
    /// 目标位置在网格外
    OutOfBounds,
    /// 目标位置不可通行
    TileNotPassable,
    /// 目标位置已被占用
    TileOccupied,
    /// 移动力不足
    InsufficientMovementPoints { required: f32, available: f32 },
    /// 路径不可达
    PathNotFound,
    /// 无效的网格坐标
    InvalidGridPosition,
}

impl std::fmt::Display for TacticalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds => write!(f, "position is out of grid bounds"),
            Self::TileNotPassable => write!(f, "target tile is not passable"),
            Self::TileOccupied => write!(f, "target tile is occupied by another unit"),
            Self::InsufficientMovementPoints {
                required,
                available,
            } => {
                write!(
                    f,
                    "insufficient MP: required={}, available={}",
                    required, available
                )
            }
            Self::PathNotFound => write!(f, "path to target not found"),
            Self::InvalidGridPosition => write!(f, "invalid grid position"),
        }
    }
}

impl std::error::Error for TacticalError {}
