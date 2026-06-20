//! 领域错误 — Terrain 域程序错误枚举。
//!
//! 涵盖地形系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `TerrainFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 地形系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"格子不可通行"）请使用 [`TerrainFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum TerrainError {
    /// 格子坐标超出地图范围。
    #[error("tile coordinates out of bounds: ({x}, {y})")]
    OutOfBounds { x: i32, y: i32 },
    /// 陷阱缺少必要的触发条件或效果定义。
    #[error("hazard zone missing required trigger or effect definition")]
    InvalidHazardDefinition,
    /// 格子 ID 未注册。
    #[error("tile not found at ({x}, {y})")]
    TileNotFound { x: i32, y: i32 },
}
