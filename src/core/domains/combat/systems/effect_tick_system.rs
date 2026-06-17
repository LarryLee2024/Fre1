//! Effect Tick System — OnTurnEnd → 效果计时推进
//!
//! 战斗领域在单位回合结束时驱动 Effect 能力领域的寿命计时与周期 Tick。
//!
//! # 架构说明
//!
//! Combat (Domain) → Effect (Capability) 方向，符合架构法第 3.2 节 "Domain 引用 Capabilities"。
//! 当天体效果（Buff/DOT/HOT）的单位回合结束信号 OnTurnEnd 被发射时，
//! 本 Observer 对所有带有 ActiveEffectContainer 的实体执行：
//! 1. 效果持续时间递减（tick_durations）
//! 2. 到期效果状态清理（expire_effects）

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::capabilities::effect::events::EffectTicked;
use crate::core::capabilities::effect::foundation::ActiveEffectContainer;
use crate::core::capabilities::effect::mechanism::{expire_effects, tick_durations};
use crate::core::domains::combat::components::TurnQueue;
use crate::core::domains::combat::events::OnTurnEnd;

/// Observer: OnTurnEnd → 推进所有实体的效果计时。
///
/// 每个单位回合结束时（OnTurnEnd），对所有 ActiveEffectContainer 执行：
/// - duration 剩余回合数 -1
/// - 周期 Tick 检测（到达 interval 时触发 Tick）
/// - Expiring → Removed 清理
pub(crate) fn on_turn_end_tick_effects(
    _trigger: On<'_, '_, OnTurnEnd>,
    mut commands: Commands,
    mut container_query: Query<&mut ActiveEffectContainer>,
    turn_queue: Res<TurnQueue>,
) {
    let current_turn = turn_queue.round_number() as u64;

    for mut container in container_query.iter_mut() {
        let result = tick_durations(&mut container, 1, current_turn);

        // 发布 Ticked 事件
        for instance_id in &result.ticked {
            if let Some(instance) = container.find_by_id(instance_id) {
                commands.trigger(EffectTicked {
                    instance_id: instance.instance_id.clone(),
                    def_id: instance.def_id.clone(),
                    target_entity: instance.target_entity.clone(),
                    tick_number: instance
                        .tick_state
                        .as_ref()
                        .map(|t| t.tick_count)
                        .unwrap_or(0),
                    total_ticks: instance.tick_state.as_ref().and_then(|t| t.max_ticks),
                });
            }
        }

        if !result.expired.is_empty() {
            debug!(
                "[Combat-Effect] {} effects expired this tick",
                result.expired.len()
            );
        }

        for (id, err) in &result.errors {
            warn!("[Combat-Effect] Tick error for '{}': {}", id, err);
        }
    }

    for mut container in container_query.iter_mut() {
        expire_effects(&mut container);
    }
}
