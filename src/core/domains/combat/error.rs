//! 领域错误 — Combat 域错误枚举。
//!
//! 涵盖战斗开始、回合流转、伤害结算、战斗结束等操作的错误。
//! 详见 docs/02-domain/domains/combat_domain.md §4

use bevy::prelude::*;

/// 战斗系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum CombatError {
    /// 参与单位不足，无法开始战斗。
    InsufficientParticipants { required: usize, actual: usize },
    /// 单位未注册为战斗参与者。
    NotCombatParticipant,
    /// 战斗尚未开始。
    CombatNotStarted,
    /// 战斗已结束，不可再操作。
    CombatAlreadyEnded,
    /// 不是该单位的回合。
    NotYourTurn,
    /// 行动资源已耗尽。
    NoActionRemaining,
    /// 先攻排序为空。
    EmptyTurnOrder,
    /// 单位已死亡，不可行动。
    UnitDead,
    /// 伤害已被结算，禁止重复结算。
    DamageAlreadyResolved,
}

impl std::fmt::Display for CombatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientParticipants { required, actual } => {
                write!(
                    f,
                    "insufficient participants: required={}, actual={}",
                    required, actual
                )
            }
            Self::NotCombatParticipant => write!(f, "entity is not a combat participant"),
            Self::CombatNotStarted => write!(f, "combat has not started"),
            Self::CombatAlreadyEnded => write!(f, "combat has already ended"),
            Self::NotYourTurn => write!(f, "it is not this unit's turn"),
            Self::NoActionRemaining => write!(f, "no action remaining this turn"),
            Self::EmptyTurnOrder => write!(f, "turn order is empty"),
            Self::UnitDead => write!(f, "unit is dead and cannot act"),
            Self::DamageAlreadyResolved => {
                write!(f, "damage already resolved, duplicate forbidden")
            }
        }
    }
}

impl std::error::Error for CombatError {}
