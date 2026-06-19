//! 规则失败 — Reaction 域业务规则不满足结果。
//!
//! 与 `ReactionError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use bevy::prelude::Entity;
use thiserror::Error;

/// 反应系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ReactionFailure {
    /// 反应槽位已用尽。
    #[error("no reactions available for reactor: {reactor:?}")]
    NoReactionsAvailable { reactor: Entity },
    /// 不在触发范围内。
    #[error("out of range: {reason}")]
    OutOfRange { reason: String },
    /// 目标不合法。
    #[error("invalid target: {reason}")]
    InvalidTarget { reason: String },
    /// 反制者法术位不足。
    #[error("insufficient spell slot for counterspell: required_level={required_level}")]
    NoCounterspellSlot { required_level: u8 },
    /// 反应类型不支持当前触发条件。
    #[error("trigger mismatch: reaction='{reaction}', trigger='{trigger}'")]
    TriggerMismatch { reaction: String, trigger: String },
}

impl RuleFailure for ReactionFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::NoReactionsAvailable { .. } => "REACTION_NO_REACTIONS_AVAILABLE",
            Self::OutOfRange { .. } => "REACTION_OUT_OF_RANGE",
            Self::InvalidTarget { .. } => "REACTION_INVALID_TARGET",
            Self::NoCounterspellSlot { .. } => "REACTION_NO_COUNTERSPELL_SLOT",
            Self::TriggerMismatch { .. } => "REACTION_TRIGGER_MISMATCH",
        }
    }
}
