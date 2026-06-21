//! Bridge — UiCommand 到 GameCommand 的接线系统
//!
//! 注册为 UiCommand 事件的 Observer，填充运行时上下文，
//! 并将生成的 GameCommand 推入 CommandQueue 供领域层执行。
//!
//! 使用 Bevy 0.19 的 Observer 模式（On<UiCommand>）而非旧版
//! EventReader 模式，与 UI 层其余部分保持一致。
//!
//! 参见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md

use bevy::ecs::observer::On;
use bevy::ecs::system::ResMut;
use tracing::info;

use crate::core::capabilities::runtime::command::foundation::CommandQueue;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::ui::application::UiCommand;

/// Observer：处理单个 UiCommand 并将对应的 GameCommand 推入 CommandQueue。
///
/// 映射规则：
/// - `EndTurn` — 以空 `unit_id` 推入；领域层负责填充
/// - `SaveGame` / `LoadGame` — 直接映射
/// - `TogglePause`, `OpenScreen`, `CloseScreen`, `NewGame` — UI 内部命令；仅记录日志
/// - 其他所有命令 — 记录为待接入领域
///
/// 每次 Observer 调用仅处理一个 UiCommand。
pub fn process_ui_commands(trigger: On<UiCommand>, mut command_queue: ResMut<CommandQueue>) {
    let cmd = trigger.event();
    match cmd {
        UiCommand::EndTurn => {
            if command_queue
                .push(GameCommand::EndTurn {
                    unit_id: String::new(),
                })
                .is_ok()
            {
                info!(target: "ui::bridge", "Command enqueued: EndTurn");
            }
        }
        UiCommand::SaveGame(_slot) => {
            if command_queue.push(GameCommand::SaveGame).is_ok() {
                info!(target: "ui::bridge", "Command enqueued: SaveGame");
            }
        }
        UiCommand::LoadGame(_slot) => {
            if command_queue.push(GameCommand::LoadGame).is_ok() {
                info!(target: "ui::bridge", "Command enqueued: LoadGame");
            }
        }
        // UI 内部命令 — 无需领域映射
        UiCommand::TogglePause => {
            info!(
                target: "ui::bridge",
                "UI-internal command (no GameCommand mapping): TogglePause",
            );
        }
        UiCommand::OpenScreen(screen) => {
            info!(
                target: "ui::bridge",
                "UI-internal command (no GameCommand mapping): OpenScreen({screen:?})",
            );
        }
        UiCommand::CloseScreen => {
            info!(
                target: "ui::bridge",
                "UI-internal command (no GameCommand mapping): CloseScreen",
            );
        }
        UiCommand::NewGame => {
            info!(
                target: "ui::bridge",
                "UI-internal command (no GameCommand mapping): NewGame",
            );
        }
        // 待接入领域的命令
        cmd => {
            info!(target: "ui::bridge", "Command pending domain wiring: {cmd:?}");
        }
    }
}
