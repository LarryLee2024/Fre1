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

/// 单位每移动一格时触发（用于逐格动画和地形效果检查）。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PositionChanged {
    pub entity: Entity,
    pub old_pos: GridPos,
    pub new_pos: GridPos,
}

/// 请求移动单位（由 GameCommand::MoveUnit 触发）。
///
/// CommandHandler 解析 GameCommand 后发出此事件，后续系统将 unit_id 解析为 Entity
/// 并触发 ComputeMoveRequest 以执行实际移动。
#[derive(Event, Debug, Clone)]
pub struct MovementRequested {
    /// 单位标识
    pub unit_id: String,
    /// 移动路径（网格坐标序列）
    pub path: Vec<GridPos>,
}

/// 请求计算移动消耗并执行移动。
///
/// Observer 将处理此事件，通过 Capabilities 管线（Tag/Attribute/Modifier）验证移动。
/// 通过 commands.trigger(ComputeMoveRequest { entity, path, .. }) 触发。
#[derive(Event, Debug, Clone)]
pub struct ComputeMoveRequest {
    /// 移动实体的 Entity
    pub entity: Entity,
    /// 移动路径（格子序列，从当前位置开始）
    pub path: Vec<GridPos>,
    /// 是否在移动后发出 UnitMoved 事件
    pub emit_moved_event: bool,
}
