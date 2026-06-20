//! 领域错误 — Economy 域程序错误枚举。
//!
//! 涵盖经济系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `EconomyFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 经济系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"余额不足"）请使用 [`EconomyFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum EconomyError {
    /// 物品不存在。
    #[error("item not found: {0}")]
    ItemNotFound(String),
}
