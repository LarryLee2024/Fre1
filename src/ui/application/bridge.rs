//! Bridge — UiCommand 到 GameCommand 的接线系统
//!
//! 注册为 UiCommand 事件的 Observer，将 UiCommand 通过
//! `into_game_command()` 转换后推入 CommandQueue 供领域层执行。
//!
//! 使用 Bevy 0.19 的 Observer 模式（On<UiCommand>）而非旧版
//! EventReader 模式，与 UI 层其余部分保持一致。
//!
//! 参见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md

use bevy::ecs::observer::On;
use bevy::ecs::system::ResMut;
use tracing::info;

use crate::core::capabilities::runtime::command::foundation::CommandQueue;
use crate::ui::application::UiCommand;

/// Observer：处理单个 UiCommand 并将对应的 GameCommand 推入 CommandQueue。
///
/// 映射规则：
/// - 所有通过 `into_game_command()` 返回 `Some` 的命令 — 自动入队
/// - `OpenScreen` / `CloseScreen` — UI 内部导航；仅记录日志，不入队
///
/// 每次 Observer 调用仅处理一个 UiCommand。
pub fn process_ui_commands(trigger: On<UiCommand>, mut command_queue: ResMut<CommandQueue>) {
    let cmd = trigger.event();

    // UI 内部导航命令 — 无需领域映射
    match cmd {
        UiCommand::OpenScreen(screen) => {
            info!(
                target: "ui::bridge",
                "UI 内部命令：OpenScreen({screen:?})",
            );
            return;
        }
        UiCommand::CloseScreen => {
            info!(target: "ui::bridge", "UI 内部命令：CloseScreen");
            return;
        }
        _ => {}
    }

    // 其余命令通过 into_game_command() 转换后入队
    if let Some(game_cmd) = cmd.into_game_command() {
        if command_queue.push(game_cmd).is_ok() {
            info!(target: "ui::bridge", "命令已入队：{cmd:?}");
        }
    }
}
