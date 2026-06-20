//! 领域错误 — Narrative 域程序错误枚举。
//!
//! 涵盖叙事系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `NarrativeFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 叙事系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"无效选择"）请使用 [`NarrativeFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum NarrativeError {
    /// 对话节点不存在。
    #[error("dialogue node not found: {node_id}")]
    DialogueNodeNotFound { node_id: String },
    /// 对话树存在循环引用。
    #[error("dialogue tree contains cycle at node: {node_id}")]
    DialogueTreeHasCycle { node_id: String },
}
