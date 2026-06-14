/// 装备模块：数据驱动的装备定义、实例管理、穿脱逻辑
/// 遵循「装备 = Modifier + Trait + Tag + Rule」四层架构
/// 支持从 content/equipments/*.ron 外部配置文件加载

/// EquipmentDef 定义与 EquipmentRegistry 注册表
mod definition;
/// 穿脱逻辑（EquipItem/UnequipItem 消息处理）
mod equip;
/// EquipmentInstance 实例管理
mod instance;
/// 装备需求检查（属性/等级/标签）
mod requirements;
/// EquipmentSlots 装备槽位组件
mod slots;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

/// 公共 re-exports
pub use definition::*;
pub use equip::*;
pub use instance::*;
pub use requirements::*;
pub use slots::*;

/// 装备插件
pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        let registry = definition::EquipmentRegistry::load_from_dir("content/equipments");
        app.insert_resource(registry)
            // 注册 Reflect 类型
            .register_type::<definition::EquipmentSlot>()
            .register_type::<definition::Rarity>()
            .register_type::<definition::EquipmentRequirement>()
            .register_type::<slots::EquipmentSlots>();
        // 注册 Message（Bevy 0.18 要求 MessageReader/Writer 前 add_message）
        app.add_message::<equip::EquipItem>();
        app.add_message::<equip::UnequipItem>();
        app.add_message::<equip::ItemEquipped>();
        app.add_message::<equip::ItemUnequipped>();
        app.add_message::<equip::EquipFailed>();
        // 注册穿脱系统：响应 EquipItem/UnequipItem Message
        app.add_systems(
            Update,
            (equip::equip_item_system, equip::unequip_item_system),
        );
    }
}
