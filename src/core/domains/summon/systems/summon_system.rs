//! 召唤 Systems
//!
//! 包括召唤物创建、消失、控制等 Observer。
//! 详见 docs/02-domain/domains/summon_domain.md §5

use bevy::prelude::*;

use super::super::components::{SummonAIMode, SummonBond, SummonSlotManager};
use super::super::events::{SummonCreated, SummonExpireReason, SummonExpired, SummonSlotChanged};
use super::super::resources::SummonConfig;
use super::super::rules::has_free_summon_slot;

/// 处理召唤物创建请求。
pub fn on_summon_created(
    _trigger: On<SummonCreated>,
    mut slot_query: Query<&mut SummonSlotManager>,
    config: Res<SummonConfig>,
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

// /// 处理召唤者死亡 — 级联消失所有召唤物。
// /// TODO: 待 UnitDied 事件接入后实现
// pub fn on_caster_died(...) {
//     // 完整实现需要：
//     // 1. 监听 UnitDied 事件
//     // 2. 查找该 Entity 的所有 SummonBond
//     // 3. 对每个召唤物触发 SummonExpired
// }
