//! 规则失败 — Narrative 域业务规则不满足结果。
//!
//! 与 `NarrativeError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

/// 叙事系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum NarrativeFailure {
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

crate::impl_rule_failure!(NarrativeFailure,
    Self::InvalidChoice { .. } => "NARRATIVE_INVALID_CHOICE",
    Self::CriticalDialogueCannotSkip => "NARRATIVE_CRITICAL_CANNOT_SKIP",
    Self::StoryFlagIrreversible { .. } => "NARRATIVE_FLAG_IRREVERSIBLE",
    Self::DialogueNotActive => "NARRATIVE_DIALOGUE_NOT_ACTIVE",
);
