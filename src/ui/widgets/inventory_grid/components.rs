//! InventoryGrid Components — Type definitions for InventoryGrid organism
//!
//! Defines the InventoryGrid marker component and InventoryGridAction enum
//! for identifying interactive elements within the grid.

use bevy::prelude::*;

/// InventoryGrid marker component
///
/// Identifies the root entity of an InventoryGrid widget.
/// Used for cleanup and query-based targeting.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct InventoryGrid;

/// Actions that can be triggered from InventoryGrid buttons
///
/// Attached as a Component to interactive child entities.
/// Observers query this component to identify which button was clicked.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum InventoryGridAction {
    /// Close the inventory screen
    Close,
}
