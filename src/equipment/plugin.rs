// 装备插件：注册 EquipmentRegistry 资源

use super::definition::EquipmentRegistry;
use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

/// 装备插件
pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        let registry = EquipmentRegistry::load_from_dir("assets/equipment");
        app.insert_resource(registry);
    }
}
