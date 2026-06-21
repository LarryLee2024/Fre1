//! 领域错误 — Summon 域程序错误枚举。
//!
//! 涵盖召唤系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `SummonFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 召唤系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"召唤槽位已满"）请使用 [`SummonFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum SummonError {
    /// 模板不存在。
    #[error("summon template 未找到: {0}")]
    TemplateNotFound(String),
}
