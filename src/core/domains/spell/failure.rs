//! 规则失败 — Spell 域业务规则不满足结果。
//!
//! 这些是正常业务结果（非程序错误），通过函数返回值传递。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use thiserror::Error;

use super::components::SpellDefId;

/// 法术系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SpellFailure {
    /// 法术位不足。
    #[error("insufficient spell slots for '{spell_id}': required_level={required_level}")]
    InsufficientSlots {
        spell_id: SpellDefId,
        required_level: u8,
    },
    /// 法术未习得。
    #[error("spell not known: {spell_id}")]
    NotKnown { spell_id: SpellDefId },
    /// 法术未准备。
    #[error("spell not prepared: {spell_id}")]
    NotPrepared { spell_id: SpellDefId },
    /// 语言成分被沉默封锁。
    #[error("verbal component blocked by silence")]
    Silenced,
    /// 姿势成分被束缚封锁。
    #[error("somatic component blocked by restraint")]
    Restrained,
    /// 材料成分缺失。
    #[error("material component missing: {description}")]
    MissingMaterial { description: String },
    /// 已有其他专注法术存在。
    #[error("already concentrating on spell: {current_spell}")]
    AlreadyConcentrating { current_spell: SpellDefId },
    /// 施法者等级不足以施放该环阶法术。
    #[error("caster level too low: required={required_level}, caster={caster_level}")]
    LevelTooLow {
        required_level: u8,
        caster_level: u8,
    },
    /// 升环施法不合法。
    #[error("invalid upcast for '{spell_id}': target_level={target_level}")]
    InvalidUpcast {
        spell_id: SpellDefId,
        target_level: u8,
    },
}

impl RuleFailure for SpellFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::InsufficientSlots { .. } => "SPELL_INSUFFICIENT_SLOTS",
            Self::NotKnown { .. } => "SPELL_NOT_KNOWN",
            Self::NotPrepared { .. } => "SPELL_NOT_PREPARED",
            Self::Silenced => "SPELL_SILENCED",
            Self::Restrained => "SPELL_RESTRAINED",
            Self::MissingMaterial { .. } => "SPELL_MISSING_MATERIAL",
            Self::AlreadyConcentrating { .. } => "SPELL_ALREADY_CONCENTRATING",
            Self::LevelTooLow { .. } => "SPELL_LEVEL_TOO_LOW",
            Self::InvalidUpcast { .. } => "SPELL_INVALID_UPCAST",
        }
    }
}
