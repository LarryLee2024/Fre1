//! InventoryScreen systems — button click handling via UiCommand routing
//!
//! Uses the ButtonClicked trigger observer with Commands::trigger
//! to map InventoryGridAction to domain commands（方案A）。

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::ui::application::UiCommand;
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::widgets::inventory_grid::components::InventoryGridAction;

/// Observer: handles inventory screen button clicks, mapping to UiCommand
///
/// When the primitives-layer `button_interaction_system` triggers a
/// `ButtonClicked` event via Commands::trigger, checks if the button
/// entity carries an `InventoryGridAction` component and dispatches the
/// corresponding UiCommand.
pub fn on_inventory_button_clicked(
    on: On<ButtonClicked>,
    query: Query<&InventoryGridAction>,
    mut commands: Commands,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        // Not an inventory button, ignore
        return;
    };

    let command = match action {
        InventoryGridAction::Close => UiCommand::CloseScreen,
    };

    info!(target: "ui", "[Inventory] 命令映射: {:?}", command);
    commands.trigger(command);
}
