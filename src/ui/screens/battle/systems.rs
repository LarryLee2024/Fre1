//! BattleScreen systems — button click handling via UiCommand routing
//!
//! Uses the ButtonClicked trigger observer with Commands::trigger
//! to map BattleAction to domain commands（方案A）。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::primitives::button::events::ButtonClicked;

/// Battle button action identifier
///
/// Attached as a Component to buttons in the battle screen. The observer
/// queries for this component to identify which button was clicked.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum BattleAction {
    /// End the current turn
    EndTurn,
}

/// Observer: handles battle button clicks, mapping to UiCommand
///
/// When the primitives-layer `button_interaction_system` triggers a
/// `ButtonClicked` event via Commands::trigger, checks if the button
/// entity carries a `BattleAction` component and dispatches the
/// corresponding UiCommand.
pub fn on_battle_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&BattleAction>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // Not a battle button, ignore
        return;
    };

    let command = match action {
        BattleAction::EndTurn => UiCommand::EndTurn,
    };

    info!(target: "ui", "[Battle] 命令映射: {:?}", command);
    commands.trigger(command);
}
