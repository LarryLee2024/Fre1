//! 领域错误 — Progression 域错误枚举
//!
//! 涵盖经验获取、升级、天赋解锁、子职选择、ASI 等操作的错误。
//! 详见 docs/02-domain/domains/progression_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

/// 成长系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum ProgressionError {
    /// 规则失败：已达到等级上限（20 级），无法继续升级。
    #[error("规则失败: max level reached")]
    MaxLevelReached,
    /// 规则失败：经验不足，无法升级。
    #[error("规则失败: insufficient experience: current={current_xp}, required={required_xp}")]
    InsufficientExperience { current_xp: u64, required_xp: u64 },
    /// 天赋前置条件不满足。
    #[error("talent prerequisite not met for '{talent_id}': {reason}")]
    TalentPrerequisiteNotMet { talent_id: String, reason: String },
    /// 该职业已有子职，不可更改。
    #[error("subclass already chosen for '{class_id}': existing={existing_subclass}")]
    SubclassAlreadyChosen {
        class_id: String,
        existing_subclass: String,
    },
    /// 规则失败：属性值已达到上限（20），无法继续提升。
    #[error("规则失败: attribute at max: {attribute}")]
    AttributeAtMax { attribute: String },
    /// 无法开始新职业：不满足属性需求。
    #[error("multiclass prerequisite not met for '{class_id}': {reason}")]
    MulticlassPrerequisiteNotMet { class_id: String, reason: String },
    /// 规则失败：ASI 不能跳过（不变量 3.5）。
    #[error("规则失败: ASI cannot be skipped at level {level}")]
    ASICannotBeSkipped { level: u32 },
}
