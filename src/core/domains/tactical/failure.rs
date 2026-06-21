//! 规则失败 — Tactical 域业务规则不满足结果。
//!
//! 这些是正常业务结果（非程序错误），通过函数返回值传递。
//! 详见 ADR-051

use thiserror::Error;

/// 战术空间业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TacticalFailure {
    /// 目标位置在网格外。
    #[error("位置超出网格边界")]
    OutOfBounds,
    /// 目标位置不可通行。
    #[error("目标格子不可通行")]
    TileNotPassable,
    /// 目标位置已被占用。
    #[error("目标格子已被其他单位占用")]
    TileOccupied,
    /// 移动力不足。
    #[error("移动力不足: required={required}, available={available}")]
    InsufficientMovementPoints { required: f32, available: f32 },
    /// 路径不可达。
    #[error("未找到通往目标的路径")]
    PathNotFound,
    /// 无效的网格坐标。
    #[error("无效的网格坐标")]
    InvalidGridPosition,
}

crate::impl_rule_failure!(TacticalFailure,
    Self::OutOfBounds => "TACTICAL_OUT_OF_BOUNDS",
    Self::TileNotPassable => "TACTICAL_TILE_NOT_PASSABLE",
    Self::TileOccupied => "TACTICAL_TILE_OCCUPIED",
    Self::InsufficientMovementPoints { .. } => "TACTICAL_INSUFFICIENT_MP",
    Self::PathNotFound => "TACTICAL_PATH_NOT_FOUND",
    Self::InvalidGridPosition => "TACTICAL_INVALID_GRID_POSITION",
);
