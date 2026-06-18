//! InventoryPlugin — 背包/物品领域 Plugin
//!
//! 注册背包组件、事件和系统。
//! 处理物品拾取、装备穿戴/卸下、消耗品使用。
//!
//! 详见 ADR-030

use bevy::prelude::*;

use super::components::{EquipmentSlots, Inventory, InventoryMarker, ItemInstance};
use super::systems::inventory_system::{on_equip_item, on_item_acquired, on_item_used};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        app.register_type::<Inventory>();
        app.register_type::<EquipmentSlots>();
        app.register_type::<ItemInstance>();
        app.register_type::<InventoryMarker>();

        // ── 注册 Observer System ──
        app.add_observer(on_item_acquired);
        app.add_observer(on_equip_item);
        app.add_observer(on_item_used);
    }
}
