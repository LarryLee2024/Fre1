//! Bridge — UiCommand to GameCommand wiring system
//!
//! Registers as an observer for UiCommand events, fills in runtime context,
//! and pushes the resulting GameCommand into the CommandQueue for domain execution.
//!
//! Uses Bevy 0.19's observer pattern (On<UiCommand>) rather than the legacy
//! EventReader pattern, consistent with the rest of the UI layer.
//!
//! See docs/01-architecture/40-cross-cutting/ADR-043-command-input.md

use bevy::ecs::observer::On;
use bevy::ecs::system::ResMut;
use tracing::info;

use crate::core::capabilities::runtime::command::foundation::CommandQueue;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::ui::application::UiCommand;

/// Observer: processes a single UiCommand and pushes the corresponding
/// GameCommand into the CommandQueue.
///
/// Mapping rules:
/// - `EndTurn` — pushed with an empty `unit_id`; the domain layer fills it in
/// - `SaveGame` / `LoadGame` — direct mapping
/// - `TogglePause`, `OpenScreen`, `CloseScreen`, `NewGame` — UI-internal; logged only
/// - All other commands — logged as pending domain wiring
///
/// Each observer invocation handles exactly one UiCommand.
pub fn process_ui_commands(
    trigger: On<UiCommand>,
    mut command_queue: ResMut<CommandQueue>,
) {
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
        // UI-internal commands — no domain mapping needed
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
        // Commands awaiting domain integration
        cmd => {
            info!(target: "ui::bridge", "Command pending domain wiring: {cmd:?}");
        }
    }
}
