//! Tactical Command Handler — handles MoveUnit GameCommand
//!
//! Listens for CommandExecuted events, matches tactical-related commands,
//! parses path coordinates, and delegates to domain events for downstream processing.

use bevy::prelude::*;
use tracing::info;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::core::domains::tactical::components::GridPos;

/// Observer: handles tactical GameCommands.
///
/// Matches `MoveUnit` variant from the command pipeline, parses path coordinates
/// into GridPos values, and emits MovementRequested for downstream movement systems.
pub fn on_tactical_command(trigger: On<CommandExecuted>, mut commands: Commands) {
    match &trigger.event().command {
        GameCommand::MoveUnit { unit_id, path } => {
            info!(target: "tactical",
                event = "command.move_unit",
                unit = %unit_id,
                path_len = path.len(),
                "MoveUnit command received"
            );

            // Parse path coordinate strings (e.g., "5,3") into GridPos values
            let coords: Vec<GridPos> = path
                .iter()
                .filter_map(|s| {
                    let parts: Vec<&str> = s.split(',').collect();
                    if parts.len() == 2 {
                        let x = parts[0].trim().parse::<i32>().ok()?;
                        let y = parts[1].trim().parse::<i32>().ok()?;
                        Some(GridPos::new(x, y))
                    } else {
                        None
                    }
                })
                .collect();

            commands.trigger(crate::core::domains::tactical::events::MovementRequested {
                unit_id: unit_id.clone(),
                path: coords,
            });
        }
        _ => {} // Not a tactical command
    }
}
