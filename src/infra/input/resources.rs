//! InputState — 当前帧输入状态（ECS Resource）
//!
//! 由 collect_input_actions System 在 PreUpdate 中更新。
//! 业务系统通过查询此 Resource 获取语义化的输入状态，而非直接读取按键。
//!
//! 详见 ADR-043 §6
//! 详见 docs/04-data/infrastructure/input_schema.md §2.5

use bevy::prelude::*;

use super::action::InputAction;

/// 当前帧输入状态 — 由 InputSystem 在 PreUpdate 中更新。
///
/// 存储经过 InputMap 翻译后的语义化输入数据和原始光标位置。
/// 业务系统（如 Tactical 域的移动选择）通过此 Resource 感知玩家输入。
#[derive(Resource, Debug, Default)]
pub struct InputState {
    /// 当前帧中按下的动作集合（持续按下）
    pub pressed_actions: Vec<InputAction>,

    /// 当前帧中刚按下的动作集合（仅按下瞬间）
    pub just_pressed_actions: Vec<InputAction>,

    /// 当前帧中刚释放的动作集合
    pub just_released_actions: Vec<InputAction>,

    /// 鼠标在屏幕上的位置（像素坐标）
    pub mouse_position: Vec2,

    /// 鼠标在网格上的位置（如果有网格上下文）
    /// 由 Tactical 域的可选 System 填充
    pub mouse_grid_pos: Option<(i32, i32)>,
}

impl InputState {
    /// 清空所有瞬时状态（每帧开始时调用）。
    pub fn clear_frame(&mut self) {
        self.just_pressed_actions.clear();
        self.just_released_actions.clear();
        self.pressed_actions.clear();
        self.mouse_grid_pos = None;
    }

    /// 检查某个动作是否在当前帧被按下。
    pub fn just_pressed(&self, action: InputAction) -> bool {
        self.just_pressed_actions.contains(&action)
    }

    /// 检查某个动作是否在当前帧刚释放。
    pub fn just_released(&self, action: InputAction) -> bool {
        self.just_released_actions.contains(&action)
    }

    /// 检查某个动作是否正在被按住。
    pub fn pressed(&self, action: InputAction) -> bool {
        self.pressed_actions.contains(&action)
    }
}
