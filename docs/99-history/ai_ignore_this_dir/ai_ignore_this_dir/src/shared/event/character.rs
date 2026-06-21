//! 角色领域事件

use crate::shared::ids::UnitId;
use bevy::prelude::*;

/// 单位已移动
#[derive(Message, Debug, Clone)]
pub struct UnitMoved {
    pub unit_id: UnitId,
    pub unit_name: String,
    pub from: (i32, i32),
    pub to: (i32, i32),
}
