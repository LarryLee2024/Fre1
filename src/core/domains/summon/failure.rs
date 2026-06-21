//! 规则失败 — Summon 域业务规则不满足结果。
//!
//! 与 `SummonError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

/// 召唤系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SummonFailure {
    /// 召唤位置不可用。
    #[error("无效的 summon 位置: {reason}")]
    InvalidPosition { reason: String },
    /// 专注冲突。
    #[error("专注冲突")]
    ConcentrationConflict,
    /// 召唤数量已达上限。
    #[error("召唤槽位已达上限: current={current}, max={max}")]
    SlotLimitReached { current: u32, max: u32 },
    /// 嵌套召唤被禁止。
    #[error("禁止嵌套召唤")]
    NestedSummonForbidden,
    /// 召唤者已死亡。
    #[error("召唤者已死亡")]
    CasterDead,
}

crate::impl_rule_failure!(SummonFailure,
    Self::InvalidPosition { .. } => "SUMMON_INVALID_POSITION",
    Self::ConcentrationConflict => "SUMMON_CONCENTRATION_CONFLICT",
    Self::SlotLimitReached { .. } => "SUMMON_SLOT_LIMIT_REACHED",
    Self::NestedSummonForbidden => "SUMMON_NESTED_FORBIDDEN",
    Self::CasterDead => "SUMMON_CASTER_DEAD",
);
