//! 规则失败 — Ability 能力层业务规则不满足结果。
//!
//! 与 `AbilityError`（程序错误）不同，这些是正常业务结果。
//! 详见 ADR-051

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AbilityFailure {
    /// 条件检查不通过
    #[error("condition check failed: {reason}")]
    ConditionFailed { reason: String },
    /// 资源消耗不足
    #[error("insufficient '{resource}': required {required}, available {available}")]
    InsufficientCost {
        resource: String,
        required: f32,
        available: f32,
    },
    /// 冷却中不可激活
    #[error("ability '{spec_id}' on cooldown ({remaining_turns} turns remaining)")]
    OnCooldown {
        spec_id: String,
        remaining_turns: u32,
    },
}

crate::impl_rule_failure!(AbilityFailure,
    Self::ConditionFailed { .. } => "ABILITY_CONDITION_FAILED",
    Self::InsufficientCost { .. } => "ABILITY_INSUFFICIENT_COST",
    Self::OnCooldown { .. } => "ABILITY_ON_COOLDOWN",
);
