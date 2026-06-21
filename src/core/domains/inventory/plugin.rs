//! InventoryPlugin — 背包/物品领域 Plugin
//!
//! 注册背包组件、事件和系统。
//! 处理物品拾取、装备穿戴/卸下、消耗品使用。
//!
//! 详见 ADR-030

use bevy::prelude::*;

use super::components::{EquipmentSlots, Inventory, InventoryMarker, ItemInstance};
use super::integration::on_inventory_command;
use super::systems::inventory_system::{on_equip_item, on_item_acquired, on_item_used};
use crate::register_domain_types;

/// 背包物品领域 Plugin——注册背包、装备槽位组件和物品操作系统。
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(
            app,
            [Inventory, EquipmentSlots, ItemInstance, InventoryMarker,]
        );

        // ── 注册 Observer System ──
        app.add_observer(on_item_acquired);
        app.add_observer(on_equip_item);
        app.add_observer(on_item_used);
        app.add_observer(on_inventory_command);
    }
}
