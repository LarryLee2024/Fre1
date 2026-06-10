pub mod battle_bag;
pub mod container;
pub mod definition;
pub mod instance;
pub mod resources;
pub mod transfer;
pub mod use_item;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;
use definition::ItemRegistry;
use instance::InstanceIdCounter;
use transfer::{ItemTransferred, TransferItem};
use use_item::{ItemUsed, UseItem};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        let registry = ItemRegistry::load_from_dir("assets/items");
        app.insert_resource(registry);
        app.insert_resource(InstanceIdCounter::default());
        // 注册 Message（Bevy 0.18 要求）
        app.add_message::<UseItem>();
        app.add_message::<ItemUsed>();
        app.add_message::<TransferItem>();
        app.add_message::<ItemTransferred>();
        // 系统
        app.add_systems(Update, use_item::use_item_system);
        app.add_systems(Update, transfer::transfer_item_system);
    }
}
