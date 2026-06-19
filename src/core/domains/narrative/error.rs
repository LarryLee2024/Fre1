//! 领域错误 — Narrative 域错误枚举。
//!
//! 涵盖对话流程、故事标记、分支选择等操作的错误。
//! 详见 docs/02-domain/domains/narrative_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

/// 叙事系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum NarrativeError {
    /// 对话节点不存在。
    #[error("dialogue node not found: {node_id}")]
    DialogueNodeNotFound { node_id: String },
    /// 对话树存在循环引用。
    #[error("dialogue tree contains cycle at node: {node_id}")]
    DialogueTreeHasCycle { node_id: String },
    /// 分支选项不在当前可选列表中。
    #[error("invalid dialogue choice: {choice_id}")]
    InvalidChoice { choice_id: String },
    /// 关键对话不可跳过。
    #[error("critical dialogue cannot be skipped")]
    CriticalDialogueCannotSkip,
    /// 故事标记试图恢复到初始值（不可逆）。
    #[error("story flag is irreversible and cannot be reverted: {flag_key}")]
    StoryFlagIrreversible { flag_key: String },
    /// 对话未开始或已结束。
    #[error("no active dialogue session")]
    DialogueNotActive,
}
