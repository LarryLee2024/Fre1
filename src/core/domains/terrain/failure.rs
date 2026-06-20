//! 规则失败 — Terrain 域业务规则不满足结果。
//!
//! 与 `TerrainError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use thiserror::Error;

/// 地形系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TerrainFailure {
    /// 格子不可通行。
    #[error("tile not passable at ({x}, {y})")]
    TileNotPassable { x: i32, y: i32 },
    /// 格子已被占用。
    #[error("tile already occupied at ({x}, {y})")]
    TileOccupied { x: i32, y: i32 },
    /// 表面类型无法通过。
    #[error("surface type not passable at ({x}, {y})")]
    SurfaceNotPassable { x: i32, y: i32 },
    /// 互斥表面类型冲突（如冰面和灼烧不可共存）。
    #[error("conflicting surface types cannot coexist on same tile")]
    ConflictingSurfaceType,
    /// 相邻格高度差超过允许的最大值。
    #[error("height difference exceeded: max_allowed={max_allowed}, actual={actual}")]
    HeightDifferenceExceeded { max_allowed: i32, actual: i32 },
    /// 无法在非陷阱格子上触发陷阱。
    #[error("no hazard at tile ({x}, {y})")]
    NoHazardAtTile { x: i32, y: i32 },
    /// 表面变化冲突。
    #[error("surface change rejected: {reason}")]
    SurfaceChangeRejected { reason: String },
}

impl RuleFailure for TerrainFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::TileNotPassable { .. } => "TERRAIN_TILE_NOT_PASSABLE",
            Self::TileOccupied { .. } => "TERRAIN_TILE_OCCUPIED",
            Self::SurfaceNotPassable { .. } => "TERRAIN_SURFACE_NOT_PASSABLE",
            Self::ConflictingSurfaceType => "TERRAIN_CONFLICTING_SURFACE",
            Self::HeightDifferenceExceeded { .. } => "TERRAIN_HEIGHT_DIFFERENCE",
            Self::NoHazardAtTile { .. } => "TERRAIN_NO_HAZARD",
            Self::SurfaceChangeRejected { .. } => "TERRAIN_SURFACE_CHANGE_REJECTED",
        }
    }
}
