//! 规则失败 — Effect 能力层业务规则不满足结果。
//!
//! 与 `EffectError`（程序错误）不同，这些是正常业务结果。
//! 详见 ADR-051

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EffectFailure {
    /// 条件不满足
    #[error("condition not met: {0}")]
    ConditionNotMet(String),
    /// 效果槽位已满
    #[error("effect slot limit reached ({current} / {max})")]
    SlotLimitReached { current: u32, max: u32 },
}

crate::impl_rule_failure!(EffectFailure,
    Self::ConditionNotMet { .. } => "EFFECT_CONDITION_NOT_MET",
    Self::SlotLimitReached { .. } => "EFFECT_SLOT_LIMIT_REACHED",
);
