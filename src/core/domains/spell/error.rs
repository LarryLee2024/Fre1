//! 领域错误 — Spell 域错误枚举
//!
//! 涵盖法术施放、法术位管理、专注维护等操作的错误。
//! 详见 docs/02-domain/domains/spell_domain.md §4

use bevy::prelude::*;

use super::components::SpellDefId;

/// 法术系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum SpellError {
    /// 法术位不足。
    InsufficientSlots {
        spell_id: SpellDefId,
        required_level: u8,
    },
    /// 法术未习得。
    SpellNotKnown { spell_id: SpellDefId },
    /// 法术未准备。
    SpellNotPrepared { spell_id: SpellDefId },
    /// 语言成分被沉默封锁。
    Silenced,
    /// 姿势成分被束缚封锁。
    Restrained,
    /// 材料成分缺失。
    MissingMaterial { description: String },
    /// 已有其他专注法术存在。
    AlreadyConcentrating { current_spell: SpellDefId },
    /// 施法者等级不足以施放该环阶法术。
    LevelTooLow {
        required_level: u8,
        caster_level: u8,
    },
    /// 法术定义未找到。
    SpellDefNotFound { spell_id: SpellDefId },
    /// 升环施法不合法。
    InvalidUpcast {
        spell_id: SpellDefId,
        target_level: u8,
    },
}
