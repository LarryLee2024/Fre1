//! InventoryGrid — 背包物品网格 Organism

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::InventoryGrid;
use self::factory::spawn_inventory_grid;

pub struct InventoryGridPlugin;

impl Plugin for InventoryGridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InventoryGrid>();
    }
}
