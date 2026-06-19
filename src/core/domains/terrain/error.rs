//! 领域错误 — Terrain 域错误枚举。
//!
//! 涵盖地形通行性、表面变化、陷阱触发等操作的错误。
//! 详见 docs/02-domain/domains/terrain_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

/// 地形系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum TerrainError {
    /// 格子坐标超出地图范围。
    #[error("tile coordinates out of bounds: ({x}, {y})")]
    OutOfBounds { x: i32, y: i32 },
    /// 格子不可通行。
    #[error("tile not passable at ({x}, {y})")]
    TileNotPassable { x: i32, y: i32 },
    /// 互斥表面类型冲突（如冰面和灼烧不可共存）。
    #[error("conflicting surface types cannot coexist on same tile")]
    ConflictingSurfaceType,
    /// 相邻格高度差超过允许的最大值。
    #[error("height difference exceeded: max_allowed={max_allowed}, actual={actual}")]
    HeightDifferenceExceeded { max_allowed: i32, actual: i32 },
    /// 陷阱缺少必要的触发条件或效果定义。
    #[error("hazard zone missing required trigger or effect definition")]
    InvalidHazardDefinition,
    /// 格子 ID 未注册。
    #[error("tile not found at ({x}, {y})")]
    TileNotFound { x: i32, y: i32 },
}
