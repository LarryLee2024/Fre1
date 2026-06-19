//! Tactical Input System — 网格光标导航与选择
//!
//! 读取 InputState，将方向键映射为光标移动，Select/Cancel 映射为确认/取消。
//! 运行在 Update 调度中。

use bevy::prelude::*;

use crate::infra::input::action::InputAction;
use crate::infra::input::resources::InputState;

use super::super::components::GridPos;
use super::super::resources::GridMap;

/// 战术域光标位置（Resource）。
///
/// 跟踪玩家在网格上的选择位置，由输入系统驱动。
#[derive(Resource, Debug, Clone)]
pub struct TacticalCursor {
    pub position: GridPos,
}

impl Default for TacticalCursor {
    fn default() -> Self {
        Self {
            position: GridPos::new(0, 0),
        }
    }
}

/// 战术域输入系统 — 处理光标移动与选择。
pub(crate) fn tactical_input_system(
    input_state: Res<InputState>,
    mut cursor: ResMut<TacticalCursor>,
    grid_map: Res<GridMap>,
) {
    let mut moved = false;

    if input_state.just_pressed(InputAction::MoveUp) {
        let new_y = cursor.position.y - 1;
        let test = GridPos::with_layer(cursor.position.x, new_y, cursor.position.layer);
        if grid_map.in_bounds(test) {
            cursor.position.y = new_y;
            moved = true;
        }
    }

    if input_state.just_pressed(InputAction::MoveDown) {
        let new_y = cursor.position.y + 1;
        let test = GridPos::with_layer(cursor.position.x, new_y, cursor.position.layer);
        if grid_map.in_bounds(test) {
            cursor.position.y = new_y;
            moved = true;
        }
    }

    if input_state.just_pressed(InputAction::MoveLeft) {
        let new_x = cursor.position.x - 1;
        let test = GridPos::with_layer(new_x, cursor.position.y, cursor.position.layer);
        if grid_map.in_bounds(test) {
            cursor.position.x = new_x;
            moved = true;
        }
    }

    if input_state.just_pressed(InputAction::MoveRight) {
        let new_x = cursor.position.x + 1;
        let test = GridPos::with_layer(new_x, cursor.position.y, cursor.position.layer);
        if grid_map.in_bounds(test) {
            cursor.position.x = new_x;
            moved = true;
        }
    }

    if moved {
        tracing::trace!(event = "tactical_input.cursor_move", pos = ?cursor.position);
    }

    if input_state.just_pressed(InputAction::Select) {
        tracing::trace!(event = "tactical_input.select", pos = ?cursor.position);
    }

    if input_state.just_pressed(InputAction::Cancel) {
        tracing::trace!(event = "tactical_input.cancel");
    }
}
