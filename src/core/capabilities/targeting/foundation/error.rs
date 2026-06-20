//! Targeting 领域错误。
//!
//! 定义目标选择与校验过程中的各类错误。
//!
//! 详见 docs/02-domain/capabilities/targeting_domain.md §1、§2。
//! 详见 docs/04-data/capabilities/targeting_schema.md §3。

use crate::core::capabilities::targeting::foundation::types::TargetType;

/// Targeting 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum TargetingError {
    /// 形状参数不合法
    InvalidShapeParameter {
        shape: String,
        param: &'static str,
        detail: String,
    },
    /// 最大目标数不合法（V2: max_targets >= 1）
    InvalidMaxTargets(u32),
    /// 射程不合法（V3: min_range <= range）
    InvalidRange {
        min: Option<f32>,
        max: Option<f32>,
        detail: String,
    },
    /// 没有合法目标
    NoValidTargets { reason: String },
    /// 目标实体不存在
    EntityNotFound(String),
    /// 超出射程
    OutOfRange { distance: f32, max_range: f32 },
    /// 阵营不匹配
    FactionMismatch {
        expected: TargetType,
        actual: String,
    },
    /// 目标数量已达上限
    TargetLimitReached { limit: u32 },
    /// 视野检查失败
    LineOfSightBlocked,
    /// 目标选择被重复调用
    AlreadySelecting,
    /// 通用运行时错误
    Runtime(String),
}

impl std::fmt::Display for TargetingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidShapeParameter {
                shape,
                param,
                detail,
            } => {
                write!(
                    f,
                    "invalid parameter '{}' for shape '{}': {}",
                    param, shape, detail
                )
            }
            Self::InvalidMaxTargets(n) => {
                write!(f, "invalid max_targets: {} (must be >= 1)", n)
            }
            Self::InvalidRange { min, max, detail } => {
                write!(
                    f,
                    "invalid range (min={:?}, max={:?}): {}",
                    min, max, detail
                )
            }
            Self::NoValidTargets { reason } => {
                write!(f, "no valid targets: {}", reason)
            }
            Self::EntityNotFound(eid) => write!(f, "entity '{}' not found", eid),
            Self::OutOfRange {
                distance,
                max_range,
            } => {
                write!(f, "distance {} exceeds max range {}", distance, max_range)
            }
            Self::FactionMismatch { expected, actual } => {
                write!(
                    f,
                    "faction mismatch: expected {:?}, got {}",
                    expected, actual
                )
            }
            Self::TargetLimitReached { limit } => {
                write!(f, "target limit of {} reached", limit)
            }
            Self::LineOfSightBlocked => write!(f, "line of sight blocked"),
            Self::AlreadySelecting => write!(f, "target selection already in progress"),
            Self::Runtime(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl std::error::Error for TargetingError {}
