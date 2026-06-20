//! BattleScreen systems — button click handling via Observer pattern
//!
//! Observers listen for `ButtonClicked` events triggered by the primitives
//! button system, then match on `BattleAction` to determine the intended action.

use bevy::ecs::observer::On;
use bevy::prelude::*;

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

/// Observer: handles battle button clicks
///
/// When the primitives-layer `button_interaction_system` triggers a
/// `ButtonClicked` event, checks if the button entity carries a
/// `BattleAction` component and dispatches accordingly.
/// Currently only logs; will be replaced by domain system integration.
pub fn on_battle_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&BattleAction>,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // Not a battle button, ignore
        return;
    };

    match action {
        BattleAction::EndTurn => {
            info!("[Battle] End Turn");
        }
    }
}
