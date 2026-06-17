//! Movement System — 移动验证与执行系统
//!
//! 处理单位移动请求：验证路径 → 消耗 MP → 更新位置 → 发布事件。

use bevy::prelude::*;

use crate::core::domains::tactical::components::{GridPos, MovementPoints};
use crate::core::domains::tactical::error::TacticalError;
use crate::core::domains::tactical::resources::GridMap;

/// 验证移动请求并执行。
///
/// 触发方式：通过 commands.trigger(MoveRequest(entity, target_pos))。
/// 实际将作为 observer 在 plugin.rs 中注册。
///
/// 当前作为纯查询函数，供更上层的 MoveSystem 使用。
pub fn validate_and_execute_move(
    _entity: Entity,
    target: GridPos,
    grid: &GridMap,
    mp: &mut MovementPoints,
    pos: &mut GridPos,
) -> Result<MoveResult, TacticalError> {
    // 1. 检查目标是否在网格内
    if !grid.in_bounds(target) {
        return Err(TacticalError::OutOfBounds);
    }

    // 2. 检查目标是否可通行
    let tile = grid
        .get_tile(target)
        .ok_or(TacticalError::InvalidGridPosition)?;
    if !tile.is_passable() {
        return Err(TacticalError::TileNotPassable);
    }

    // 3. 计算移动消耗（简化：每格 1 MP）
    let distance = pos.manhattan_distance(target) as f32;
    let cost = distance;

    // 4. 检查 MP 是否足够
    if cost > mp.current {
        return Err(TacticalError::InsufficientMovementPoints {
            required: cost,
            available: mp.current,
        });
    }

    // 5. 执行移动
    let old_pos = *pos;
    mp.consume(cost);
    *pos = target;

    Ok(MoveResult {
        old_pos,
        new_pos: target,
        cost,
        remaining_mp: mp.current,
    })
}

/// 移动执行结果。
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub old_pos: GridPos,
    pub new_pos: GridPos,
    pub cost: f32,
    pub remaining_mp: f32,
}
