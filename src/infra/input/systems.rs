//! Input Systems — 输入采集与命令入队
//!
//! 在 PreUpdate 中运行：
//!   1. collect_input_actions: 读取原始按键 → InputMap 翻译 → 更新 InputState
//!   2. process_meta_commands:  将无上下文的元命令（Save/Load/Menu）入队到 CommandQueue
//!
//! 有上下文的业务命令（MoveUnit/EndTurn/Attack 等）由各 Domain 的系统自行处理：
//!   Domain 系统读取 InputState + 当前游戏状态 → 构造 GameCommand → CommandQueue.push()
//!
//! 详见 ADR-043 §4
//! 详见 docs/04-data/infrastructure/input_schema.md

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::foundation::{
    CommandQueue, CommandSource, GameCommand,
};

use super::action::{InputAction, InputMap};
use super::resources::InputState;

/// 采集原始输入并翻译为语义化 InputAction，更新 InputState。
///
/// 在 PreUpdate 中第一个执行。读取 Bevy 的 ButtonInput 并通过 InputMap 翻译。
pub fn collect_input_actions(
    input_map: Res<InputMap>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut input_state: ResMut<InputState>,
) {
    // 清空上一帧的瞬时状态
    input_state.clear_frame();

    // 采集键盘按键
    for (key, action) in &input_map.keyboard {
        if keyboard.pressed(*key) {
            input_state.pressed_actions.push(*action);
        }
        if keyboard.just_pressed(*key) {
            input_state.just_pressed_actions.push(*action);
        }
        if keyboard.just_released(*key) {
            input_state.just_released_actions.push(*action);
        }
    }

    // 采集鼠标按键
    for (button, action) in &input_map.mouse {
        if mouse_buttons.pressed(*button) && !input_state.pressed_actions.contains(action) {
            input_state.pressed_actions.push(*action);
        }
        if mouse_buttons.just_pressed(*button) && !input_state.just_pressed_actions.contains(action)
        {
            input_state.just_pressed_actions.push(*action);
        }
        if mouse_buttons.just_released(*button)
            && !input_state.just_released_actions.contains(action)
        {
            input_state.just_released_actions.push(*action);
        }
    }

    tracing::trace!(
        "Input: 按下={:?} 刚按下={:?}",
        input_state.pressed_actions,
        input_state.just_pressed_actions
    );
}

/// 处理无上下文的元命令（QuickSave/QuickLoad/OpenMenu）。
///
/// 这些命令不依赖游戏状态，可以直接在 infra 层入队。
/// 业务命令（MoveUnit/EndTurn/Attack 等）由各 Domain 的 System 自行处理。
pub fn process_meta_commands(
    input_state: Res<InputState>,
    mut command_queue: ResMut<CommandQueue>,
) {
    for action in &input_state.just_pressed_actions {
        match action {
            InputAction::QuickSave => {
                let _ = command_queue.push_recorded(GameCommand::SaveGame, CommandSource::Player);
                tracing::info!("[Input] QuickSave 命令已入队");
            }
            InputAction::QuickLoad => {
                let _ = command_queue.push_recorded(GameCommand::LoadGame, CommandSource::Player);
                tracing::info!("[Input] QuickLoad 命令已入队");
            }
            InputAction::OpenMenu => {
                let _ = command_queue.push_recorded(GameCommand::OpenMenu, CommandSource::Player);
                tracing::info!("[Input] OpenMenu 命令已入队");
            }
            _ => {
                // 其他动作（方向键、技能槽、Select/Cancel 等）由 Domain 层处理
            }
        }
    }
}
