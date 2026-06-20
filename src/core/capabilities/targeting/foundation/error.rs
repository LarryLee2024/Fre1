//! Targeting 领域错误。
//!
//! 定义目标选择与校验过程中的各类错误。
//!
//! 详见 docs/02-domain/capabilities/targeting_domain.md §1、§2。
//! 详见 docs/04-data/capabilities/targeting_schema.md §3。

use crate::core::capabilities::targeting::foundation::types::TargetType;
use thiserror::Error;

/// Targeting 领域错误。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TargetingError {
    /// 形状参数不合法
    #[error("invalid parameter '{param}' for shape '{shape}': {detail}")]
    InvalidShapeParameter {
        shape: String,
        param: &'static str,
        detail: String,
    },
    /// 最大目标数不合法（V2: max_targets >= 1）
    #[error("invalid max_targets: {max} (must be >= 1)")]
    InvalidMaxTargets { max: u32 },
    /// 射程不合法（V3: min_range <= range）
    #[error("invalid range (min={min:?}, max={max:?}): {detail}")]
    InvalidRange {
        min: Option<f32>,
        max: Option<f32>,
        detail: String,
    },
    /// 没有合法目标
    #[error("no valid targets: {reason}")]
    NoValidTargets { reason: String },
    /// 目标实体不存在
    #[error("entity '{entity_id}' not found")]
    EntityNotFound { entity_id: String },
    /// 超出射程
    #[error("distance {distance} exceeds max range {max_range}")]
    OutOfRange { distance: f32, max_range: f32 },
    /// 阵营不匹配
    #[error("faction mismatch: expected {expected:?}, got {actual}")]
    FactionMismatch {
        expected: TargetType,
        actual: String,
    },
    /// 目标数量已达上限
    #[error("target limit of {limit} reached")]
    TargetLimitReached { limit: u32 },
    /// 视野检查失败
    #[error("line of sight blocked")]
    LineOfSightBlocked,
    /// 目标选择被重复调用
    #[error("target selection already in progress")]
    AlreadySelecting,
}
