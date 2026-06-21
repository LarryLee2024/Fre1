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
    #[error("shape '{shape}' 的参数 '{param}' 不合法: {detail}")]
    InvalidShapeParameter {
        shape: String,
        param: &'static str,
        detail: String,
    },
    /// 最大目标数不合法（V2: max_targets >= 1）
    #[error("无效的 max_targets: {max}（必须 >= 1）")]
    InvalidMaxTargets { max: u32 },
    /// 射程不合法（V3: min_range <= range）
    #[error("无效的 range (min={min:?}, max={max:?}): {detail}")]
    InvalidRange {
        min: Option<f32>,
        max: Option<f32>,
        detail: String,
    },
    /// 没有合法目标
    #[error("没有合法目标: {reason}")]
    NoValidTargets { reason: String },
    /// 目标实体不存在
    #[error("entity '{entity_id}' 未找到")]
    EntityNotFound { entity_id: String },
    /// 超出射程
    #[error("距离 {distance} 超过最大射程 {max_range}")]
    OutOfRange { distance: f32, max_range: f32 },
    /// 阵营不匹配
    #[error("faction 不匹配: 期望 {expected:?}, 实际 {actual}")]
    FactionMismatch {
        expected: TargetType,
        actual: String,
    },
    /// 目标数量已达上限
    #[error("目标数量上限 {limit} 已满")]
    TargetLimitReached { limit: u32 },
    /// 视野检查失败
    #[error("视线被阻挡")]
    LineOfSightBlocked,
    /// 目标选择被重复调用
    #[error("目标选择正在进行中")]
    AlreadySelecting,
}
