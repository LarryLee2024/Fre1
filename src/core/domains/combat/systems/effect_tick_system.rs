//! Effect Tick System — OnTurnEnd → 效果计时推进
//!
//! 战斗领域在单位回合结束时驱动 Effect 能力领域的寿命计时与周期 Tick。
//!
//! # 架构说明
//!
//! Combat (Domain) → Effect (Capability) 方向，符合架构法第 3.2 节 "Domain 引用 Capabilities"。
//! 所有 Effect Capabilities 的交互通过 `super::integration::effect/` 模块完成，
//! 禁止直接 import `ActiveEffectContainer` / `tick_durations` / `expire_effects`。
//!
//! 详见 ADR-024。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::domains::combat::events::OnTurnEnd;
use crate::core::domains::combat::integration::effect::EffectTickParam;

use crate::infra::logging::rate_limit::OnceGuard;

// ── Rate limiting guards for high-frequency warn! calls ──
static EFFECT_TICK_ERRORS_GUARD: OnceGuard = OnceGuard::new();

/// Observer: OnTurnEnd → 推进所有实体的效果计时。
///
/// 每个单位回合结束时（OnTurnEnd），对所有 ActiveEffectContainer 执行：
/// - duration 剩余回合数 -1
/// - 周期 Tick 检测（到达 interval 时触发 Tick）
/// - Expiring → Removed 清理
///
/// 通过 EffectTickParam（integration/effect/system_param.rs）与 Effect Capability 交互，
/// 不直接接触 Capabilities 内部类型。
#[tracing::instrument(skip_all, target = "combat")]
pub(crate) fn on_turn_end_tick_effects(
    _trigger: On<'_, '_, OnTurnEnd>,
    mut commands: Commands,
    mut effect_tick: EffectTickParam,
) {
    let outcomes = effect_tick.tick_all(&mut commands);

    // 记录 Tick 活动日志
    for outcome in &outcomes {
        if !outcome.ticked.is_empty() {
            debug!(target: "combat",
                "[Combat-Effect] {} 个效果触发 Tick，{} 个到期",
                outcome.ticked.len(),
                outcome.expired.len()
            );
        }

        if outcome.error_count > 0 {
            if EFFECT_TICK_ERRORS_GUARD.try_fire() {
                tracing::warn!(target: "combat",
                event = "combat.effect_tick.errors",
                error_count = outcome.error_count,
                "Tick 过程中出现 {} 个错误",
                outcome.error_count
            );
            }
        }
    }
}
