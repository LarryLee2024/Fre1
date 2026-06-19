//! 领域错误 — Spell 域错误枚举
//!
//! 涵盖法术施放、法术位管理、专注维护等操作的错误。
//! 详见 docs/02-domain/domains/spell_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

use super::components::SpellDefId;

/// 法术系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum SpellError {
    /// 规则失败：法术位不足。
    #[error("规则失败: insufficient spell slots for '{spell_id}': required_level={required_level}")]
    InsufficientSlots {
        spell_id: SpellDefId,
        required_level: u8,
    },
    /// 法术未习得。
    #[error("spell not known: {spell_id}")]
    SpellNotKnown { spell_id: SpellDefId },
    /// 法术未准备。
    #[error("spell not prepared: {spell_id}")]
    SpellNotPrepared { spell_id: SpellDefId },
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
    /// 法术定义未找到。
    #[error("spell definition not found: {spell_id}")]
    SpellDefNotFound { spell_id: SpellDefId },
    /// 升环施法不合法。
    #[error("invalid upcast for '{spell_id}': target_level={target_level}")]
    InvalidUpcast {
        spell_id: SpellDefId,
        target_level: u8,
    },
}
