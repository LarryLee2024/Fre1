//! 领域错误 — Narrative 域错误枚举。
//!
//! 涵盖对话流程、故事标记、分支选择等操作的错误。
//! 详见 docs/02-domain/domains/narrative_domain.md §4

use bevy::prelude::*;

/// 叙事系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum NarrativeError {
    /// 对话节点不存在。
    DialogueNodeNotFound { node_id: String },
    /// 对话树存在循环引用。
    DialogueTreeHasCycle { node_id: String },
    /// 分支选项不在当前可选列表中。
    InvalidChoice { choice_id: String },
    /// 关键对话不可跳过。
    CriticalDialogueCannotSkip,
    /// 故事标记试图恢复到初始值（不可逆）。
    StoryFlagIrreversible { flag_key: String },
    /// 对话未开始或已结束。
    DialogueNotActive,
}

impl std::fmt::Display for NarrativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DialogueNodeNotFound { node_id } => {
                write!(f, "dialogue node not found: {}", node_id)
            }
            Self::DialogueTreeHasCycle { node_id } => {
                write!(f, "dialogue tree contains cycle at node: {}", node_id)
            }
            Self::InvalidChoice { choice_id } => {
                write!(f, "invalid dialogue choice: {}", choice_id)
            }
            Self::CriticalDialogueCannotSkip => {
                write!(f, "critical dialogue cannot be skipped")
            }
            Self::StoryFlagIrreversible { flag_key } => {
                write!(
                    f,
                    "story flag is irreversible and cannot be reverted: {}",
                    flag_key
                )
            }
            Self::DialogueNotActive => write!(f, "no active dialogue session"),
        }
    }
}

impl std::error::Error for NarrativeError {}
