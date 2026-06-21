//! 领域错误 — Party 域程序错误枚举。
//!
//! 涵盖队伍系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `PartyFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 队伍系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"队伍已满"）请使用 [`PartyFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum PartyError {
    /// 未找到指定成员。
    #[error("成员未找到: {entity}")]
    MemberNotFound { entity: Entity },
    /// 羁绊模板未注册。
    #[error("bond definition 未找到: {bond_id}")]
    BondDefNotFound { bond_id: String },
}
