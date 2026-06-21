//! 领域错误 — Combat 域程序错误枚举。
//!
//! 涵盖战斗系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `CombatFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 战斗系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"不是你的回合"）请使用 [`CombatFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CombatError {
    /// 单位未注册为战斗参与者。
    #[error("entity 不是战斗参与者")]
    NotCombatParticipant,
    /// 战斗尚未开始。
    #[error("战斗尚未开始")]
    CombatNotStarted,
    /// 战斗已结束，不可再操作。
    #[error("战斗已结束")]
    CombatAlreadyEnded,
    /// 先攻排序为空。
    #[error("回合顺序为空")]
    EmptyTurnOrder,
    /// 伤害已被结算，禁止重复结算。
    #[error("伤害已结算，禁止重复结算")]
    DamageAlreadyResolved,
}
