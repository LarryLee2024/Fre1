// 装备插件：注册 EquipmentRegistry 资源 + Message

use super::definition::EquipmentRegistry;
use super::equip::{EquipFailed, EquipItem, ItemEquipped, ItemUnequipped, UnequipItem};
use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

/// 装备插件
pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        let registry = EquipmentRegistry::load_from_dir("assets/equipment");
        app.insert_resource(registry);
        // 注册 Message（Bevy 0.18 要求 MessageReader/Writer 前 add_message）
        app.add_message::<EquipItem>();
        app.add_message::<UnequipItem>();
        app.add_message::<ItemEquipped>();
        app.add_message::<ItemUnequipped>();
        app.add_message::<EquipFailed>();
    }
}
