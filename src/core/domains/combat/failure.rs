//! 规则失败 — Combat 域业务规则不满足结果。
//!
//! 与 `CombatError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 docs/02-domain/domains/combat_domain.md §4

use thiserror::Error;

/// 战斗系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CombatFailure {
    /// 参与单位不足，无法开始战斗。
    #[error("参与者不足: 需要={required}, 实际={actual}")]
    InsufficientParticipants { required: usize, actual: usize },
    /// 不是该单位的回合。
    #[error("不是该单位的回合")]
    NotYourTurn,
    /// 行动资源已耗尽。
    #[error("本回合无剩余行动")]
    NoActionRemaining,
    /// 单位已死亡，不可行动。
    #[error("单位已死亡，无法行动")]
    UnitDead,
}

crate::impl_rule_failure!(CombatFailure,
    Self::InsufficientParticipants { .. } => "COMBAT_INSUFFICIENT_PARTICIPANTS",
    Self::NotYourTurn => "COMBAT_NOT_YOUR_TURN",
    Self::NoActionRemaining => "COMBAT_NO_ACTION",
    Self::UnitDead => "COMBAT_UNIT_DEAD",
);
