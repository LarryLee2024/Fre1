//! 召唤 Systems
//!
//! 包括召唤物创建、消失、控制等 Observer。
//! 详见 docs/02-domain/domains/summon_domain.md §5

use bevy::prelude::*;

use super::super::components::{SummonAIMode, SummonBond, SummonSlotManager};
use super::super::events::{SummonCommand, SummonCreated, SummonExpired, SummonSlotChanged};
use super::super::resources::SummonConfig;
use super::super::rules::has_free_summon_slot;

/// 处理召唤物创建请求。
pub fn on_summon_created(
    _trigger: On<SummonCreated>,
    mut slot_query: Query<&mut SummonSlotManager>,
    _config: Res<SummonConfig>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if let Ok(mut manager) = slot_query.get_mut(event.caster) {
        if !has_free_summon_slot(&manager) {
            return;
        }
        manager.add_summon(event.summon_entity);

        commands.trigger(SummonSlotChanged {
            caster: event.caster,
            slots_used: manager.active_summons.len() as u32,
            slots_max: manager.max_slots,
        });
    }
}

/// 处理召唤物消失请求。
pub fn on_summon_expired(
    _trigger: On<SummonExpired>,
    mut slot_query: Query<&mut SummonSlotManager>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if let Ok(mut manager) = slot_query.get_mut(event.caster) {
        manager.remove_summon(event.summon_entity);

        commands.trigger(SummonSlotChanged {
            caster: event.caster,
            slots_used: manager.active_summons.len() as u32,
            slots_max: manager.max_slots,
        });
    }
}

/// 创建召唤物并触发 SummonCreated 事件。
///
/// 执行完整的创建流程：生成实体 → 附加 SummonBond → 触发事件。
pub fn handle_summon_created(
    mut commands: Commands,
    caster: Entity,
    template_id: String,
    position: (i32, i32),
    duration_type: String,
) -> Entity {
    // 创建召唤物实体
    let summon_entity = commands.spawn_empty().id();

    // 附加 SummonBond 组件
    commands.entity(summon_entity).insert(SummonBond {
        caster,
        template_id: template_id.clone(),
        ai_mode: SummonAIMode::Follow,
        summoned_at: 0.0,
    });

    commands.trigger(SummonCreated {
        caster,
        summon_entity,
        template_id,
        position,
        duration_type,
    });

    summon_entity
}

/// 移除召唤物并触发 SummonExpired 事件。
pub fn handle_summon_expired(
    mut commands: Commands,
    caster: Entity,
    summon_entity: Entity,
    reason: super::super::events::SummonExpireReason,
) {
    commands.trigger(SummonExpired {
        caster,
        summon_entity,
        reason,
    });

    // 销毁召唤物实体
    commands.entity(summon_entity).despawn();
}

/// 发出召唤物指令并触发 SummonCommand 事件。
pub fn handle_summon_command(
    mut commands: Commands,
    caster: Entity,
    summon_entity: Entity,
    command_type: super::super::events::SummonCommandType,
) {
    commands.trigger(SummonCommand {
        caster,
        summon_entity,
        command_type,
    });
}

// TODO(on_caster_died): 召唤者死亡级联消失 — 已记录至 docs/09-planning/ 跟踪项
// 实现要点: 监听 UnitDied → 查找 SummonBond → 逐个触发 SummonExpired
