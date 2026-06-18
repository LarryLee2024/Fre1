//! 领域错误 — Reaction 域错误枚举
//!
//! 涵盖反应触发、执行、反制判定等操作的错误。
//! 详见 docs/02-domain/domains/reaction_domain.md §4

use bevy::prelude::*;

/// 反应系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ReactionError {
    /// 反应槽位已用尽。
    NoReactionsAvailable { reactor: Entity },
    /// 不在触发范围内（援护距离检查）。
    OutOfRange { reason: String },
    /// 目标不合法。
    InvalidTarget { reason: String },
    /// 反制者法术位不足。
    NoCounterspellSlot { required_level: u8 },
    /// 反应类型不支持当前触发条件。
    TriggerMismatch { reaction: String, trigger: String },
    /// 特殊反应未注册。
    SpecialNotRegistered { custom_id: String },
}
