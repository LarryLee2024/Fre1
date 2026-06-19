//! 领域错误 — Combat 域错误枚举。
//!
//! 涵盖战斗开始、回合流转、伤害结算、战斗结束等操作的错误。
//! 详见 docs/02-domain/domains/combat_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

/// 战斗系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CombatError {
    /// 参与单位不足，无法开始战斗。
    #[error("insufficient participants: required={required}, actual={actual}")]
    InsufficientParticipants { required: usize, actual: usize },
    /// 单位未注册为战斗参与者。
    #[error("entity is not a combat participant")]
    NotCombatParticipant,
    /// 战斗尚未开始。
    #[error("combat has not started")]
    CombatNotStarted,
    /// 战斗已结束，不可再操作。
    #[error("combat has already ended")]
    CombatAlreadyEnded,
    /// 不是该单位的回合。
    #[error("it is not this unit's turn")]
    NotYourTurn,
    /// 行动资源已耗尽。
    #[error("no action remaining this turn")]
    NoActionRemaining,
    /// 先攻排序为空。
    #[error("turn order is empty")]
    EmptyTurnOrder,
    /// 单位已死亡，不可行动。
    #[error("unit is dead and cannot act")]
    UnitDead,
    /// 伤害已被结算，禁止重复结算。
    #[error("damage already resolved, duplicate forbidden")]
    DamageAlreadyResolved,
}
