//! EffectFacade — 效果能力的战斗域语义 API。
//!
//! 所有 Capabilities 内部类型（`ActiveEffectContainer`、`TickResult` 等）的字段访问
//! 都在此文件中完成。Systems 和 Rules 通过视图类型交互，永远不直接访问 Effect Capability 内部。
//!
//! # 职责边界
//!
//! - ✅ 封装 `tick_durations` + `expire_effects` 为 Combat 域的业务语义 API
//! - ✅ 将 `TickResult` 转换为 Combat 域视图类型 `EffectTickOutcome`
//! - ✅ 合并 tick + expire 为单次 pass（解决 Debt-D9-002）
//! - 🟥 禁止：在此模块之外直接 import `ActiveEffectContainer` / `tick_durations` / `expire_effects`

use bevy::prelude::*;

use crate::core::capabilities::effect::foundation::ActiveEffectContainer;
use crate::core::capabilities::effect::mechanism::{expire_effects, tick_durations};

use super::types::EffectTickOutcome;

// ─── ReadFacade ────────────────────────────────────────────────────────

/// 检查实体是否有活跃效果。
pub fn has_active_effects(container: &ActiveEffectContainer) -> bool {
    container.effects.iter().any(|e| e.stage.is_active())
}

/// 检查实体是否有指定 `def_id` 的活跃效果。
pub fn has_active_effect_by_def(container: &ActiveEffectContainer, def_id: &str) -> bool {
    container
        .effects
        .iter()
        .any(|e| e.def_id == def_id && e.stage.is_active())
}

/// 统计活跃效果数量。
pub fn active_effect_count(container: &ActiveEffectContainer) -> usize {
    container
        .effects
        .iter()
        .filter(|e| e.stage.is_active())
        .count()
}

// ─── WriteFacade（Combat 域唯一调用 tick_durations / expire_effects 的地方） ───

/// 推进所有效果的持续时间计时（单次 tick）。
///
/// 封装 `tick_durations`，将 `TickResult` 转换为 Combat 域视图 `EffectTickOutcome`。
/// 仅执行计时推进，不清理到期效果——调用方需要决定何时调用 `expire_all_effects`。
pub fn tick_all_effects(
    container: &mut ActiveEffectContainer,
    current_turn: u64,
    commands: &mut Commands,
) -> EffectTickOutcome {
    // 推进 1 回合（OnTurnEnd 语义）
    let result = tick_durations(container, 1, current_turn, commands);

    EffectTickOutcome {
        ticked: result.ticked,
        expired: result.expired,
        error_count: result.errors.len(),
    }
}

/// 清理所有已到期的效果（Expiring → Removed）。
///
/// 封装 `expire_effects`。必须在 `tick_all_effects` 之后调用，
/// 否则 Expiring 效果无法被清理。
pub fn expire_all_effects(container: &mut ActiveEffectContainer) -> Vec<String> {
    expire_effects(container)
}

/// 推进计时 + 清理到期效果，一步到位。
///
/// 等同于 `tick_all_effects` + `expire_all_effects` 串行执行，
/// 但返回的 `EffectTickOutcome` 包含两者合并的结果。
///
/// # 设计决策
///
/// 此函数解决 Debt-D9-002：原 `effect_tick_system.rs` 对 `Query` 做了两次 `.iter_mut()`
/// 遍历（一次 tick_durations，一次 expire_effects）。通过在此处合并为单次 pass，
/// Observer 只需一次 Query 遍历。
pub fn tick_and_expire(
    container: &mut ActiveEffectContainer,
    current_turn: u64,
    commands: &mut Commands,
) -> EffectTickOutcome {
    let outcome = tick_all_effects(container, current_turn, commands);
    let _expired_ids = expire_all_effects(container);
    outcome
}

/// 重置效果暂停状态（如回合开始时恢复暂停的效果）。
pub fn resume_all_effects(container: &mut ActiveEffectContainer) {
    for effect in &mut container.effects {
        if effect.paused && effect.stage.is_active() {
            effect.paused = false;
        }
    }
}
