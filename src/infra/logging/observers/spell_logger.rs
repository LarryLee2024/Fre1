//! spell_logger — 法术事件日志 Observer
//!
//! 监听法术域事件，生成 INFO 日志。
//! 领域层不写日志，由本模块通过 Observer 生成。

use bevy::prelude::*;

use crate::core::domains::spell::events::SpellCastResult;
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 法术施放结果日志 Observer。
///
/// 监听 `SpellCastResult` 事件，记录施法者和施法结果。
#[tracing::instrument(skip_all, fields(code = ?LogCode::SPR001, event = "法术施放"))]
pub(crate) fn on_spell_cast_result(trigger: On<SpellCastResult>) {
    metrics::record(LogCode::SPR001);
    let event = trigger.event();
    info!(
        code = ?LogCode::SPR001,
        event = "法术施放",
        caster = ?event.caster,
        spell_id = ?event.spell_id,
        result = ?event.result,
        "法术施放"
    );
}
