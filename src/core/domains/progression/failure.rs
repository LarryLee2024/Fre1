//! 规则失败 — Progression 域业务规则不满足结果。
//!
//! 与 `ProgressionError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

/// 成长系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ProgressionFailure {
    /// 已达到等级上限（20 级），无法继续升级。
    #[error("已达最高等级")]
    MaxLevelReached,
    /// 经验不足，无法升级。
    #[error("经验不足: current={current_xp}, required={required_xp}")]
    InsufficientExperience { current_xp: u64, required_xp: u64 },
    /// 天赋前置条件不满足。
    #[error("talent '{talent_id}' 前置条件不满足: {reason}")]
    TalentPrerequisiteNotMet { talent_id: String, reason: String },
    /// 该职业已有子职，不可更改。
    #[error("'{class_id}' 已有子职: existing={existing_subclass}")]
    SubclassAlreadyChosen {
        class_id: String,
        existing_subclass: String,
    },
    /// 属性值已达到上限（20），无法继续提升。
    #[error("属性已达上限: {attribute}")]
    AttributeAtMax { attribute: String },
    /// 无法开始新职业：不满足属性需求。
    #[error("'{class_id}' 兼职前置条件不满足: {reason}")]
    MulticlassPrerequisiteNotMet { class_id: String, reason: String },
    /// ASI 不能跳过。
    #[error("ASI 在 {level} 级不可跳过")]
    ASICannotBeSkipped { level: u32 },
}

crate::impl_rule_failure!(ProgressionFailure,
    Self::MaxLevelReached => "PROGRESSION_MAX_LEVEL",
    Self::InsufficientExperience { .. } => "PROGRESSION_INSUFFICIENT_XP",
    Self::TalentPrerequisiteNotMet { .. } => "PROGRESSION_TALENT_PREREQUISITE",
    Self::SubclassAlreadyChosen { .. } => "PROGRESSION_SUBCLASS_CHOSEN",
    Self::AttributeAtMax { .. } => "PROGRESSION_ATTRIBUTE_AT_MAX",
    Self::MulticlassPrerequisiteNotMet { .. } => "PROGRESSION_MULTICLASS_PREREQUISITE",
    Self::ASICannotBeSkipped { .. } => "PROGRESSION_ASI_CANNOT_SKIP",
);
