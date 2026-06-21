//! Module Name: InventoryGrid Widget — Inventory item grid organism
//!
//! Composes Panel / Text / InventoryItemRow / Button into a structured
//! inventory grid view with title, gold display, item list, and close button.
//! Registered as a sub-plugin of WidgetsPlugin.

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{InventoryGrid, InventoryGridAction};

/// InventoryGridPlugin — registers InventoryGrid component types
///
/// Added by WidgetsPlugin. No update systems needed as this is a
/// static layout composition of existing widgets.
pub struct InventoryGridPlugin;

impl Plugin for InventoryGridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InventoryGrid>()
            .register_type::<InventoryGridAction>();
    }
}
