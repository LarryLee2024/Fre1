//! 召唤 Systems
//!
//! 包括召唤物创建、消失、控制等 Observer。
//! 详见 docs/02-domain/domains/summon_domain.md §5

use bevy::prelude::*;

use super::super::components::SummonSlotManager;
use super::super::events::{SummonCreated, SummonExpired, SummonSlotChanged};
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

// TODO(on_caster_died): 召唤者死亡级联消失 — 已记录至 docs/09-planning/ 跟踪项
// 实现要点: 监听 UnitDied → 查找 SummonBond → 逐个触发 SummonExpired
