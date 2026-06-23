//! OpenMenu — 处理 GameCommand::OpenMenu → UiCommand::OpenScreen(Settings) 桥接
//!
//! 当玩家按下 ESC 键时，输入系统将 KeyCode::Escape 映射为 InputAction::OpenMenu，
//! 经过 collect_input_actions → process_meta_commands 转换为 GameCommand::OpenMenu，
//! 标记为 Dispatched 后触发 CommandExecuted 事件。
//!
//! 本 Observer 消费 CommandExecuted(OpenMenu) 事件，触发 UiCommand::OpenScreen(Settings)，
//! 由 UI 层的 on_open_settings_screen observer 生成设置屏幕。
//!
//! 关联需求：menu-flow-plan.md P0 #4 — ESC→Settings 快捷键

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::ui::application::UiCommand;
use crate::ui::navigation::ScreenType;

/// Observer: 处理 GameCommand::OpenMenu → 打开设置屏幕
///
/// 当命令处理系统（command_processing_system）发出 CommandExecuted
/// 且 command 为 GameCommand::OpenMenu 时，触发 UiCommand::OpenScreen(Settings)。
///
/// 此模式与 game_setup::on_new_game_command 处理 NewGame → PartySetup 的流程一致。
pub fn on_open_menu_command(trigger: On<CommandExecuted>, mut commands: Commands) {
    if matches!(&trigger.event().command, GameCommand::OpenMenu) {
        info!(
            target: "app",
            "OpenMenu command executed — opening Settings overlay",
        );
        commands.trigger(UiCommand::OpenScreen(ScreenType::Settings));
    }
}
