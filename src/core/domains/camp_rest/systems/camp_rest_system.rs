//! 营地/休息管理 Systems
//!
//! 包括短休/长休处理、中断管理、营地事件触发等 System。

use bevy::prelude::*;

use crate::core::domains::camp_rest::components::{CampEventDef, HitDicePool, RestState};
use crate::core::domains::camp_rest::events::{
    LongRestCompleted, LongRestInterrupted, ShortRestCompleted,
};
use crate::core::domains::camp_rest::rules::can_trigger_camp_event;
use crate::shared::time::GameTime;

/// 短休完成处理 System。
///
/// 响应短休完成，消耗生命骰，重置休息状态。
/// HP 恢复效果由订阅 ShortRestCompleted 事件的消费者处理。
pub fn handle_short_rest_complete(
    trigger: On<ShortRestCompleted>,
    mut pool_query: Query<&mut HitDicePool>,
    mut rest_query: Query<&mut RestState>,
) {
    for entity in &trigger.event().entities {
        // 消耗生命骰
        if let Ok(mut pool) = pool_query.get_mut(*entity) {
            pool.spend(trigger.event().hit_dice_used);
        }

        // 重置休息状态
        if let Ok(mut rest) = rest_query.get_mut(*entity) {
            rest.reset();
        }
    }
}

/// 长休完成处理 System。
///
/// 响应长休完成，恢复生命骰，重置休息状态。
/// HP 全恢复和法术位恢复由订阅 LongRestCompleted 事件的消费者处理。
pub fn handle_long_rest_complete(
    trigger: On<LongRestCompleted>,
    mut pool_query: Query<&mut HitDicePool>,
    mut rest_query: Query<&mut RestState>,
    game_time: Res<GameTime>,
) {
    for entity in &trigger.event().entities {
        // 恢复生命骰（不变量 3.4）
        if let Ok(mut pool) = pool_query.get_mut(*entity) {
            pool.recover_for_long_rest();
        }

        // 记录上次长休时间，然后重置休息状态
        if let Ok(mut rest) = rest_query.get_mut(*entity) {
            rest.last_long_rest_frame = Some(game_time.frame());
            rest.reset();
        }
    }
}

/// 长休中断处理 System。
///
/// 不变量 3.5：中断累计超过 1 小时导致长休失败。
pub fn handle_long_rest_interrupted(
    trigger: On<LongRestInterrupted>,
    mut rest_query: Query<&mut RestState>,
) {
    for entity in &trigger.event().entities {
        if let Ok(mut rest) = rest_query.get_mut(*entity) {
            rest.fail(); // 标记休息失败
        }
    }
}

/// 营地事件触发 System（占位）。
///
/// 当前为简化实现，完整的营地事件系统依赖 Narrative 领域就绪。
pub fn process_camp_events(rest_query: Query<&RestState>) {
    // 简化实现：完整的营地事件触发逻辑待 Narrative 领域就绪后实现
    // 当前为占位
    for rest in rest_query.iter() {
        if can_trigger_camp_event(rest) {
            // TODO: 触发营地事件
        }
    }
}

/// 营地事件注册 Resource（占位）。
#[derive(Resource, Debug, Clone)]
pub struct CampEventRegistry {
    pub events: Vec<CampEventDef>,
}

impl Default for CampEventRegistry {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}
