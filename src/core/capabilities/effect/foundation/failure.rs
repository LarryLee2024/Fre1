//! 规则失败 — Effect 能力层业务规则不满足结果。
//!
//! 与 `EffectError`（程序错误）不同，这些是正常业务结果。
//! 详见 ADR-051

use thiserror::Error;

/// Effect 业务规则失败——正常的效果施加条件不满足结果，非程序错误。
///
/// 与 `EffectError` 的区别：EffectError 表示不应发生的异常，EffectFailure 是合法的业务拒绝。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EffectFailure {
    /// 条件不满足
    #[error("条件不满足: {0}")]
    ConditionNotMet(String),
    /// 效果槽位已满
    #[error("effect 槽位已达上限 ({current} / {max})")]
    SlotLimitReached { current: u32, max: u32 },
}

crate::impl_rule_failure!(EffectFailure,
    Self::ConditionNotMet { .. } => "EFFECT_CONDITION_NOT_MET",
    Self::SlotLimitReached { .. } => "EFFECT_SLOT_LIMIT_REACHED",
);
