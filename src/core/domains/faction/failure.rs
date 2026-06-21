//! 规则失败 — Faction 域业务规则不满足结果。
//!
//! 与 `FactionError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

use super::components::FactionId;

/// 阵营系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FactionFailure {
    /// 声望值超出有效范围 [-100, +100]。
    #[error("声望值超出范围: {value}")]
    ReputationOutOfRange { value: i32 },
    /// 实体不属于指定阵营。
    #[error("entity 不是 faction {faction_id} 的成员")]
    NotMemberOfFaction { faction_id: FactionId },
    /// 关键角色声望保护触发，不允许降到该阈值以下。
    #[error("faction {faction_id} 的关键角色声望保护已触发")]
    CriticalCharacterProtection { faction_id: FactionId },
}

crate::impl_rule_failure!(FactionFailure,
    Self::ReputationOutOfRange { .. } => "FACTION_REPUTATION_OUT_OF_RANGE",
    Self::NotMemberOfFaction { .. } => "FACTION_NOT_MEMBER",
    Self::CriticalCharacterProtection { .. } => "FACTION_CRITICAL_PROTECTION",
);
