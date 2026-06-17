//! API — Combat 域公开查询接口
//!
//! 供外部在遵守 Data Law 012（域间禁止直接数据引用）的前提下，通过函数调用而非
//! 直接字段访问来获取 combat 域的状态信息。
//!
//! # 可用 API
//!
//! | 函数 | 返回值 | 说明 |
//! |------|--------|------|
//! | `get_current_turn` | `Option<TurnEntry>` | 当前回合的行动单位（克隆值） |
//! | `get_turn_queue_info` | `TurnQueueInfo` | 回合队列的快照信息 |

use bevy::prelude::*;

use super::components::{TurnEntry, TurnQueue};

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
