//! 领域错误 — Faction 域程序错误枚举。
//!
//! 涵盖阵营系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `FactionFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

use super::components::FactionId;

/// 阵营系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"声望超出范围"）请使用 [`FactionFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum FactionError {
    /// 阵营 ID 未注册。
    #[error("faction not found: {faction_id}")]
    FactionNotFound { faction_id: FactionId },
    /// 阵营间关系不对称，违反对称性不变量。
    #[error("faction relation asymmetry detected between {a} and {b}")]
    RelationAsymmetry { a: FactionId, b: FactionId },
}
