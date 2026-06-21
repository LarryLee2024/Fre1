//! 规则失败 — CampRest 域业务规则不满足结果。
//!
//! 与 `CampRestError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

/// 营地/休息系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CampRestFailure {
    /// 战斗状态中无法休息。
    #[error("战斗中无法休息")]
    InCombat,
    /// 不在安全区域。
    #[error("不在安全区域")]
    NotSafe,
    /// 24 小时内已进行过长休。
    #[error("24 小时内已休息过")]
    AlreadyRestedWithin24h,
    /// 长休被中断超过 1 小时。
    #[error("长休被打断: cumulative_minutes={cumulative_minutes}")]
    InterruptedTimeout { cumulative_minutes: u32 },
    /// 生命骰不足。
    #[error("hit dice 不足: 可用={available}, 需求={requested}")]
    InsufficientHitDice { available: u32, requested: u32 },
    /// 当前休息阶段不允许该操作。
    #[error("无效的 rest phase: 当前={current_phase}, 期望={expected}")]
    InvalidPhase {
        current_phase: String,
        expected: String,
    },
}

crate::impl_rule_failure!(CampRestFailure,
    Self::InCombat => "CAMPREST_IN_COMBAT",
    Self::NotSafe => "CAMPREST_NOT_SAFE",
    Self::AlreadyRestedWithin24h => "CAMPREST_ALREADY_RESTED_24H",
    Self::InterruptedTimeout { .. } => "CAMPREST_INTERRUPTED_TIMEOUT",
    Self::InsufficientHitDice { .. } => "CAMPREST_INSUFFICIENT_HIT_DICE",
    Self::InvalidPhase { .. } => "CAMPREST_INVALID_PHASE",
);
