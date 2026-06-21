//! 规则失败 — Ability 能力层业务规则不满足结果。
//!
//! 与 `AbilityError`（程序错误）不同，这些是正常业务结果。
//! 详见 ADR-051

use thiserror::Error;

/// Ability 业务规则失败——正常的技能激活条件不满足结果，非程序错误。
///
/// 与 `AbilityError` 的区别：AbilityError 表示不应发生的异常，AbilityFailure 是合法的业务拒绝。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AbilityFailure {
    /// 条件检查不通过
    #[error("条件检查失败: {reason}")]
    ConditionFailed { reason: String },
    /// 资源消耗不足
    #[error("'{resource}' 不足: 需要 {required}, 可用 {available}")]
    InsufficientCost {
        resource: String,
        required: f32,
        available: f32,
    },
    /// 冷却中不可激活
    #[error("ability '{spec_id}' 冷却中（剩余 {remaining_turns} 回合）")]
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
