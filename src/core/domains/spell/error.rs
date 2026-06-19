//! 领域错误 — Spell 域程序错误枚举
//!
//! 仅包含程序错误（程序缺陷或环境问题），与 `SpellFailure`（规则失败）严格区分。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

use super::components::SpellDefId;

/// 法术系统程序错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum SpellError {
    /// 法术定义未找到。
    #[error("spell definition not found: {spell_id}")]
    SpellDefNotFound { spell_id: SpellDefId },
}
