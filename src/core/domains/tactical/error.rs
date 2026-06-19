//! 领域错误 — Tactical 域专属错误枚举。
//!
//! 遵循宪法 §8：分领域错误枚举，禁止全局 AppError。

use bevy::prelude::*;
use thiserror::Error;

/// Tactical 领域错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum TacticalError {
    /// 目标位置在网格外
    #[error("position is out of grid bounds")]
    OutOfBounds,
    /// 目标位置不可通行
    #[error("target tile is not passable")]
    TileNotPassable,
    /// 目标位置已被占用
    #[error("target tile is occupied by another unit")]
    TileOccupied,
    /// 移动力不足
    #[error("insufficient MP: required={required}, available={available}")]
    InsufficientMovementPoints { required: f32, available: f32 },
    /// 路径不可达
    #[error("path to target not found")]
    PathNotFound,
    /// 无效的网格坐标
    #[error("invalid grid position")]
    InvalidGridPosition,
}
