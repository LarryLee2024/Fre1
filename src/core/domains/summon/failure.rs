//! 规则失败 — Summon 域业务规则不满足结果。
//!
//! 与 `SummonError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use thiserror::Error;

/// 召唤系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SummonFailure {
    /// 召唤位置不可用。
    #[error("invalid summon position: {reason}")]
    InvalidPosition { reason: String },
    /// 专注冲突。
    #[error("concentration conflict")]
    ConcentrationConflict,
    /// 召唤数量已达上限。
    #[error("summon slot limit reached: current={current}, max={max}")]
    SlotLimitReached { current: u32, max: u32 },
    /// 嵌套召唤被禁止。
    #[error("nested summon forbidden")]
    NestedSummonForbidden,
    /// 召唤者已死亡。
    #[error("caster is dead")]
    CasterDead,
}

impl RuleFailure for SummonFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::InvalidPosition { .. } => "SUMMON_INVALID_POSITION",
            Self::ConcentrationConflict => "SUMMON_CONCENTRATION_CONFLICT",
            Self::SlotLimitReached { .. } => "SUMMON_SLOT_LIMIT_REACHED",
            Self::NestedSummonForbidden => "SUMMON_NESTED_FORBIDDEN",
            Self::CasterDead => "SUMMON_CASTER_DEAD",
        }
    }
}
