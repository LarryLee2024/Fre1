//! 回合领域事件

use bevy::prelude::*;

/// 回合已开始
#[derive(Message, Debug, Clone)]
pub struct TurnStarted {
    pub turn_number: u32,
    pub faction: String,
}

/// 回合已结束
#[derive(Message, Debug, Clone)]
pub struct TurnEnded {
    pub turn_number: u32,
    pub next_faction: String,
}
