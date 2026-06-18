//! 反应管理 Systems
//!
//! 包括反应触发处理、队列管理和回合重置等 System。
//! 详见 docs/02-domain/domains/reaction_domain.md §5

use bevy::prelude::*;

use super::super::components::ReactionState;
use super::super::events::{OpportunityAttackExecuted, ReactionExecuted, ReactionTriggered};
use super::super::resources::GlobalReactionQueue;

/// 回合开始时重置反应槽位。
///
/// 每回合开始时，将所有单位的反应状态重置为可用。
pub fn reset_reactions_on_turn_start(mut query: Query<&mut ReactionState>) {
    for mut state in &mut query {
        state.reset();
    }
}

/// 处理反应队列中的条目。
///
/// 从全局反应队列中取出下一个待处理的条目，检查其可用性：
/// - 如果可用，发布 ReactionTriggered 事件供下游决策，然后返回（一次只处理一个）
/// - 如果不可用，跳过此条目，继续查找下一个
pub fn process_reaction_queue(
    mut commands: Commands,
    mut queue: ResMut<GlobalReactionQueue>,
    query: Query<&ReactionState>,
) {
    while !queue.queue.is_finished() {
        // 查找下一个 Pending 条目
        let entry = match queue.queue.next_pending() {
            Some(e) => e.clone(),
            None => break,
        };

        // 检查触发者是否仍可用反应
        if let Ok(state) = query.get(entry.reactor) {
            if state.can_react() {
                // 可用 → 触发事件，一次只处理一个反应
                let ctx = format!("{:?} 触发 {:?}", entry.reactor, entry.reaction_type);
                commands.trigger(ReactionTriggered {
                    reactor: entry.reactor,
                    reaction_type: entry.reaction_type,
                    priority: entry.priority,
                    context: ctx,
                });
                return;
            }
        }

        // 不可用 → 跳过此条目，继续查找下一个
        queue.queue.cancel_current();
    }
}

/// 清理已完成的反应队列（帧末执行）。
pub fn cleanup_reaction_queue(mut queue: ResMut<GlobalReactionQueue>) {
    if queue.queue.is_finished() {
        queue.clear();
    }
}

/// 监听机会攻击执行事件，转发 ReactionExecuted。
pub fn on_opportunity_attack_executed(
    _trigger: On<OpportunityAttackExecuted>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    let result = if event.hit {
        format!(
            "机会攻击命中，造成 {} 点伤害{}",
            event.damage,
            if event.critical {
                "（重击！）"
            } else {
                ""
            }
        )
    } else {
        "机会攻击未命中".to_string()
    };

    commands.trigger(ReactionExecuted {
        reactor: event.attacker,
        reaction_type: super::super::components::ReactionType::OpportunityAttack,
        result,
    });
}
