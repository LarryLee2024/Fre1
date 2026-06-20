//! spell_logger — 法术事件日志 Observer
//!
//! 监听法术域事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::spell::events::SpellCastResult;
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 法术施放结果日志 Observer。
#[tracing::instrument(skip_all, target = "domain.spell", fields(
    code = ?LogCode::SPR001,
    event = "spell_cast",
))]
pub(crate) fn on_spell_cast_result(trigger: On<SpellCastResult>) {
    metrics::record(LogCode::SPR001);
    let event = trigger.event();
    info!(
        target = "domain.spell",
        caster = ?event.caster,
        spell_id = ?event.spell_id,
        result = ?event.result,
        "法术施放",
    );
}
