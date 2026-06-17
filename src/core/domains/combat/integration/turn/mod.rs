//! Turn Queue Facade — 回合队列的跨域查询接口。
//!
//! 供外部在遵守 Data Law 012（域间禁止直接数据引用）的前提下，
//! 通过函数调用而非直接字段访问来获取 Combat 域的回合状态信息。
//!
//! # 可用 API
//!
//! | 函数 | 返回值 | 说明 |
//! |------|--------|------|
//! | `get_current_turn` | `Option<TurnEntry>` | 当前回合的行动单位（克隆值） |
//! | `get_turn_queue_info` | `TurnQueueInfo` | 回合队列的快照信息 |
//! | `mark_unit_dead` | — | 标记单位阵亡 |

use bevy::prelude::*;

use crate::core::domains::combat::components::{CombatParticipant, TurnEntry, TurnQueue};

/// 回合队列的快照信息（值类型，不暴露内部引用）。
#[derive(Debug, Clone, PartialEq)]
pub struct TurnQueueInfo {
    pub current_entity: Option<Entity>,
    pub current_team: Option<String>,
    pub round_number: u32,
    pub total_units: usize,
    pub current_index: usize,
}

/// 获取当前行动的单位条目（克隆数据以避免生命周期约束）。
pub fn get_current_turn(turn_queue: &TurnQueue) -> Option<TurnEntry> {
    turn_queue.current().cloned()
}

/// 标记某单位的战斗参与者为阵亡。
///
/// 由死亡系统（或 HP ≤ 0 时的处理系统）调用。
/// 胜利条件检查会根据此标记判定团队是否全灭。
pub fn mark_unit_dead(participant: &mut CombatParticipant) {
    participant.is_alive = false;
}

/// 获取回合队列的快照信息。
pub fn get_turn_queue_info(turn_queue: &TurnQueue) -> TurnQueueInfo {
    TurnQueueInfo {
        current_entity: turn_queue.current().map(|e| e.entity),
        current_team: turn_queue.current_team().map(|t| t.0.clone()),
        round_number: turn_queue.round_number(),
        total_units: turn_queue.len(),
        current_index: turn_queue.current_index(),
    }
}
