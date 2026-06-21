//! Quest Command Handler — handles AcceptQuest/AbandonQuest GameCommands
//!
//! Listens for CommandExecuted events and delegates to quest domain events.
//! Follows the same pattern as inventory/economy command handlers.

use bevy::prelude::*;
use tracing::info;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::core::domains::quest::events::{QuestAbandoned, QuestAcceptRequested};

/// Observer: handles quest GameCommands.
///
/// Matches `AcceptQuest` and `AbandonQuest` variants from the command
/// pipeline and emits domain-specific events for existing quest systems.
pub fn on_quest_command(trigger: On<CommandExecuted>, mut commands: Commands) {
    match &trigger.event().command {
        GameCommand::AcceptQuest {
            unit_id,
            quest_def_id,
        } => {
            info!(target: "quest",
                event = "command.accept_quest",
                unit = %unit_id,
                quest = %quest_def_id,
                "AcceptQuest command received"
            );
            commands.trigger(QuestAcceptRequested {
                unit_id: unit_id.clone(),
                quest_def_id: quest_def_id.clone(),
            });
        }
        GameCommand::AbandonQuest {
            unit_id,
            quest_def_id,
        } => {
            info!(target: "quest",
                event = "command.abandon_quest",
                unit = %unit_id,
                quest = %quest_def_id,
                "AbandonQuest command received"
            );
            commands.trigger(QuestAbandoned {
                unit_id: unit_id.clone(),
                quest_def_id: quest_def_id.clone(),
            });
        }
        _ => {} // Not a quest command
    }
}
