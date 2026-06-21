//! 规则失败 — Spell 域业务规则不满足结果。
//!
//! 这些是正常业务结果（非程序错误），通过函数返回值传递。
//! 详见 ADR-051

use thiserror::Error;

use super::components::SpellDefId;

/// 法术系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SpellFailure {
    /// 法术位不足。
    #[error("'{spell_id}' 法术位不足: required_level={required_level}")]
    InsufficientSlots {
        spell_id: SpellDefId,
        required_level: u8,
    },
    /// 法术未习得。
    #[error("法术未习得: {spell_id}")]
    NotKnown { spell_id: SpellDefId },
    /// 法术未准备。
    #[error("法术未准备: {spell_id}")]
    NotPrepared { spell_id: SpellDefId },
    /// 语言成分被沉默封锁。
    #[error("语言成分被沉默封锁")]
    Silenced,
    /// 姿势成分被束缚封锁。
    #[error("姿势成分被束缚封锁")]
    Restrained,
    /// 材料成分缺失。
    #[error("材料成分缺失: {description}")]
    MissingMaterial { description: String },
    /// 已有其他专注法术存在。
    #[error("已在专注法术: {current_spell}")]
    AlreadyConcentrating { current_spell: SpellDefId },
    /// 施法者等级不足以施放该环阶法术。
    #[error("施法者等级不足: required={required_level}, caster={caster_level}")]
    LevelTooLow {
        required_level: u8,
        caster_level: u8,
    },
    /// 升环施法不合法。
    #[error("'{spell_id}' 升环不合法: target_level={target_level}")]
    InvalidUpcast {
        spell_id: SpellDefId,
        target_level: u8,
    },
}

crate::impl_rule_failure!(SpellFailure,
    Self::InsufficientSlots { .. } => "SPELL_INSUFFICIENT_SLOTS",
    Self::NotKnown { .. } => "SPELL_NOT_KNOWN",
    Self::NotPrepared { .. } => "SPELL_NOT_PREPARED",
    Self::Silenced => "SPELL_SILENCED",
    Self::Restrained => "SPELL_RESTRAINED",
    Self::MissingMaterial { .. } => "SPELL_MISSING_MATERIAL",
    Self::AlreadyConcentrating { .. } => "SPELL_ALREADY_CONCENTRATING",
    Self::LevelTooLow { .. } => "SPELL_LEVEL_TOO_LOW",
    Self::InvalidUpcast { .. } => "SPELL_INVALID_UPCAST",
);
