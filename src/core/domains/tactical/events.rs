//! 领域事件 — Tactical 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。

use bevy::prelude::*;

use super::components::GridPos;

/// 单位完成移动时触发（整个移动过程结束）。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct UnitMoved {
    pub entity: Entity,
    pub from: GridPos,
    pub to: GridPos,
    pub remaining_mp: f32,
}

/// 单位每移动一格时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PositionChanged {
    pub entity: Entity,
    pub old_pos: GridPos,
    pub new_pos: GridPos,
}
