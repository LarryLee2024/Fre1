//! 战役/关卡领域事件

use bevy::prelude::*;

/// 关卡已完成
#[derive(Message, Debug, Clone)]
pub struct LevelCompleted {
    pub level_id: String,
    pub success: bool,
    pub turns_used: u32,
}
