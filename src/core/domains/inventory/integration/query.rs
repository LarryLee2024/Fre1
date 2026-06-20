//! InventoryQueryParam — Bevy SystemParam，封装 Inventory 域查询。
//!
//! Systems 通过此 param 查询背包/装备数据，完全不知道组件内部细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(inv: InventoryQueryParam) {
//!     if let Some(inventory) = inv.get_inventory(entity) {
//!         // ...
//!     }
//!     if inv.has_inventory_marker(entity) {
//!         // ...
//!     }
//! }
//! ```

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::inventory::components::{
    EquipSlot, EquipmentSlots, Inventory, InventoryMarker, ItemInstance,
};

use super::facade::InventoryReadFacade;

/// Inventory 查询参数 — 封装所有 Inventory 域组件查询。
///
/// System 签名中使用此类型以访问 Inventory 域数据。
/// 函数体内所有查询都通过此 param 的方法完成。
///
/// 字段为 `pub` 以便高级用法直接访问原始查询迭代器，
/// 但应优先使用本类型提供的封装方法。
#[derive(SystemParam)]
pub struct InventoryQueryParam<'w, 's> {
    /// 背包组件查询。
    pub inventory_query: Query<'w, 's, &'static Inventory>,
    /// 装备槽位组件查询。
    pub equipment_query: Query<'w, 's, &'static EquipmentSlots>,
    /// 物品实例组件查询。
    pub item_instance_query: Query<'w, 's, &'static ItemInstance>,
    /// 背包标记组件查询。
    pub marker_query: Query<'w, 's, &'static InventoryMarker>,
}

impl<'w, 's> InventoryQueryParam<'w, 's> {
    /// 获取实体的背包组件。
    pub fn get_inventory(&self, entity: Entity) -> Option<&Inventory> {
        self.inventory_query.get(entity).ok()
    }

    /// 获取实体的装备槽位组件。
    pub fn get_equipment_slots(&self, entity: Entity) -> Option<&EquipmentSlots> {
        self.equipment_query.get(entity).ok()
    }

    /// 获取实体的物品实例组件。
    pub fn get_item_instance(&self, entity: Entity) -> Option<&ItemInstance> {
        self.item_instance_query.get(entity).ok()
    }

    /// 检查实体是否有背包标记。
    pub fn has_inventory_marker(&self, entity: Entity) -> bool {
        self.marker_query.get(entity).is_ok()
    }

    /// 获取实体背包中指定模板 ID 的物品。
    pub fn find_item_in_inventory(
        &self,
        entity: Entity,
        template_id: &str,
    ) -> Option<&ItemInstance> {
        self.inventory_query
            .get(entity)
            .ok()?
            .find_item(template_id)
    }

    /// 检查实体背包中是否有指定物品（及数量）。
    pub fn has_item(&self, entity: Entity, template_id: &str, quantity: u32) -> bool {
        self.inventory_query
            .get(entity)
            .ok()
            .is_some_and(|inv| inv.has_item(template_id, quantity))
    }

    /// 获取装备槽位中指定位置的装备。
    pub fn get_equipped_item(&self, entity: Entity, slot: EquipSlot) -> Option<&ItemInstance> {
        self.equipment_query.get(entity).ok()?.get(slot)
    }

    /// 检查指定装备槽位是否为空。
    pub fn is_equip_slot_empty(&self, entity: Entity, slot: EquipSlot) -> bool {
        self.equipment_query
            .get(entity)
            .ok()
            .is_some_and(|slots| slots.is_slot_empty(slot))
    }
}
