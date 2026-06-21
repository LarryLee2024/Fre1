//! 规则失败 — Party 域业务规则不满足结果。
//!
//! 与 `PartyError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use bevy::prelude::Entity;
use thiserror::Error;

/// 队伍系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PartyFailure {
    /// 队伍已满，无法添加新成员。
    #[error("party full: current={current}, max={max}")]
    Full { current: usize, max: u32 },
    /// 该成员已在队伍中。
    #[error("entity already in party: {entity:?}")]
    AlreadyInParty { entity: Entity },
    /// 活跃成员已满，无法激活更多。
    #[error("active party full: current={current}, max={max}")]
    ActiveFull { current: usize, max: u32 },
    /// 本回合已进行过换人操作。
    #[error("swap already performed this turn")]
    SwapAlreadyPerformedThisTurn,
    /// 行动力不足，无法换人。
    #[error("insufficient action points for swap: required={required}, available={available}")]
    InsufficientActionPoints { required: u32, available: u32 },
    /// 该队员不在预备队伍中。
    #[error("entity not in reserve: {entity:?}")]
    NotInReserve { entity: Entity },
    /// 羁绊已激活。
    #[error("bond already active: {bond_id}")]
    BondAlreadyActive { bond_id: String },
}

crate::impl_rule_failure!(PartyFailure,
    Self::Full { .. } => "PARTY_FULL",
    Self::AlreadyInParty { .. } => "PARTY_ALREADY_IN_PARTY",
    Self::ActiveFull { .. } => "PARTY_ACTIVE_FULL",
    Self::SwapAlreadyPerformedThisTurn => "PARTY_SWAP_ALREADY_PERFORMED",
    Self::InsufficientActionPoints { .. } => "PARTY_INSUFFICIENT_AP",
    Self::NotInReserve { .. } => "PARTY_NOT_IN_RESERVE",
    Self::BondAlreadyActive { .. } => "PARTY_BOND_ALREADY_ACTIVE",
);
