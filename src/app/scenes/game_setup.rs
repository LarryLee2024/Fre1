//! Game Setup — handles NewGame command to PartySetup state transition
//!
//! Observes CommandExecuted events and transitions to PartySetup
//! when a GameCommand::NewGame is processed by the command system.
//!
//! See ADR-050: StateTransitionQueue + TransitionRequest.

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::shared::game_state::TransitionRequest;

use super::queue::StateTransitionQueue;

/// Observer: handles GameCommand::NewGame → transition to PartySetup
///
/// When the command processing system emits CommandExecuted with
/// GameCommand::NewGame, this observer enqueues a Change(PartySetup)
/// request via StateTransitionQueue (the only permitted path for
/// GameState transitions per ADR-050).
pub fn on_new_game_command(trigger: On<CommandExecuted>, mut queue: ResMut<StateTransitionQueue>) {
    if matches!(&trigger.event().command, GameCommand::NewGame) {
        info!(target: "app", "NewGame command executed — transitioning to PartySetup");
        queue.push(TransitionRequest::Change(
            crate::shared::game_state::GameState::PartySetup,
        ));
    }
}
