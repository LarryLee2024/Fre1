//! Module Name: InventoryScreen — Inventory management screen
//!
//! Full-screen inventory view composed of InventoryGrid widget.
//! Uses primitives-layer factories and widget factories exclusively.
//! No direct Node/Button/Interaction manipulation outside factories.
//!
//! UI tree structure:
//!
//! ```text
//! Panel (Basic, full screen, centered)
//!   └── InventoryGrid
//!         ├── Text ("Inventory", Heading)
//!         ├── Text ("Gold: 100", Caption)
//!         ├── InventoryItemRow × N
//!         └── Button ("Close", Secondary)
//! ```

pub mod systems;

use bevy::prelude::*;

use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::theme::Theme;
use crate::ui::widgets::inventory_grid::factory::spawn_inventory_grid;

/// Inventory screen marker component
///
/// Used for scene-management cleanup (despawn all entities carrying this
/// component when leaving the inventory screen).
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct InventoryScreen;

/// Startup System: spawns the inventory screen
///
/// Creates the full-screen inventory UI tree. All elements are created
/// through primitives/widget factories -- no direct Node/Button/Interaction
/// manipulation.
pub fn spawn_inventory_screen(
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    // ── 1. Root panel (full screen, centered) ──
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        InventoryScreen,
    ));

    // ── 2. Inventory grid ──
    let grid = spawn_inventory_grid(&mut commands, &asset_server, &theme);
    commands.entity(grid).set_parent_in_place(root);
}

/// 清除系统：离开背包时销毁所有背包屏幕实体
pub fn despawn_inventory_screen(
    mut commands: Commands,
    query: Query<Entity, With<InventoryScreen>>,
) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
