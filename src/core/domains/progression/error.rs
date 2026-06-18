//! 领域错误 — Progression 域错误枚举
//!
//! 涵盖经验获取、升级、天赋解锁、子职选择、ASI 等操作的错误。
//! 详见 docs/02-domain/domains/progression_domain.md §4

use bevy::prelude::*;

/// 成长系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ProgressionError {
    /// 已达到等级上限（20 级），无法继续升级。
    MaxLevelReached,
    /// 经验不足，无法升级。
    InsufficientExperience { current_xp: u64, required_xp: u64 },
    /// 天赋前置条件不满足。
    TalentPrerequisiteNotMet { talent_id: String, reason: String },
    /// 该职业已有子职，不可更改。
    SubclassAlreadyChosen {
        class_id: String,
        existing_subclass: String,
    },
    /// 属性值已达到上限（20），无法继续提升。
    AttributeAtMax { attribute: String },
    /// 无法开始新职业：不满足属性需求。
    MulticlassPrerequisiteNotMet { class_id: String, reason: String },
    /// ASI 不能跳过（不变量 3.5）。
    ASICannotBeSkipped { level: u32 },
}
