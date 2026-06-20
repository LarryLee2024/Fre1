//! Facade — Inventory 域只读/写入外观层。
//!
//! 其他域通过此 Facade 访问 Inventory 域组件数据，不直接引用组件类型。
//!
//! # 使用示例
//!
//! ```rust,ignore
//! // 只读查询
//! let inventory = InventoryReadFacade::get_inventory(world, entity);
//!
//! // 写入修改
//! InventoryWriteFacade::add_item_to_inventory(world, entity, item, 3.0);
//! ```

use bevy::prelude::*;

use crate::core::domains::inventory::components::{
    EquipSlot, EquipmentSlots, Inventory, InventoryMarker, ItemInstance,
};

// ─── Read Facade ─────────────────────────────────────────────────────

/// Inventory 只读 Facade。
///
/// 提供对 Inventory 域组件数据的只读查询方法。
/// 所有方法接受 `&World`，返回 `Option<&T>`。
pub struct InventoryReadFacade;

impl InventoryReadFacade {
    /// 获取实体的背包组件。
    pub fn get_inventory(world: &World, entity: Entity) -> Option<&Inventory> {
        world.get::<Inventory>(entity)
    }

    /// 获取实体的装备槽位组件。
    pub fn get_equipment_slots(world: &World, entity: Entity) -> Option<&EquipmentSlots> {
        world.get::<EquipmentSlots>(entity)
    }

    /// 获取实体的物品实例组件。
    pub fn get_item_instance(world: &World, entity: Entity) -> Option<&ItemInstance> {
        world.get::<ItemInstance>(entity)
    }

    /// 检查实体是否拥有 InventoryMarker。
    pub fn has_inventory_marker(world: &World, entity: Entity) -> bool {
        world.get::<InventoryMarker>(entity).is_some()
    }

    /// 获取实体背包中指定模板 ID 的物品。
    pub fn find_item_in_inventory<'w>(
        world: &'w World,
        entity: Entity,
        template_id: &str,
    ) -> Option<&'w ItemInstance> {
        world.get::<Inventory>(entity)?.find_item(template_id)
    }

    /// 检查实体背包中是否有指定模板 ID 的物品（及数量）。
    pub fn inventory_has_item(
        world: &World,
        entity: Entity,
        template_id: &str,
        quantity: u32,
    ) -> bool {
        world
            .get::<Inventory>(entity)
            .is_some_and(|inv| inv.has_item(template_id, quantity))
    }

    /// 检查背包是否还有空位容纳物品。
    pub fn can_hold_in_inventory(
        world: &World,
        entity: Entity,
        item: &ItemInstance,
        weight: f32,
    ) -> bool {
        world
            .get::<Inventory>(entity)
            .is_some_and(|inv| inv.can_hold(item, weight))
    }

    /// 获取装备槽位中指定位置的装备。
    pub fn get_equipped_item(
        world: &World,
        entity: Entity,
        slot: EquipSlot,
    ) -> Option<&ItemInstance> {
        world.get::<EquipmentSlots>(entity)?.get(slot)
    }

    /// 检查指定装备槽位是否为空。
    pub fn is_equip_slot_empty(world: &World, entity: Entity, slot: EquipSlot) -> bool {
        world
            .get::<EquipmentSlots>(entity)
            .is_some_and(|slots| slots.is_slot_empty(slot))
    }
}

// ─── Write Facade ────────────────────────────────────────────────────

/// Inventory 写入 Facade。
///
/// 提供对 Inventory 域组件数据的写入修改方法。
/// 所有方法接受 `&mut World`，无业务逻辑，仅纯数据操作。
pub struct InventoryWriteFacade;

impl InventoryWriteFacade {
    /// 添加物品到实体背包。返回实际添加的数量。
    pub fn add_item_to_inventory(
        world: &mut World,
        entity: Entity,
        item: ItemInstance,
        weight: f32,
    ) -> u32 {
        let Ok(mut entity_mut) = world.get_entity_mut(entity) else {
            return 0;
        };
        let Some(mut inventory) = entity_mut.get_mut::<Inventory>() else {
            return 0;
        };
        inventory.add_item(item, weight)
    }

    /// 从实体背包移除物品。返回实际移除的数量。
    pub fn remove_item_from_inventory(
        world: &mut World,
        entity: Entity,
        template_id: &str,
        quantity: u32,
        weight_per_unit: f32,
    ) -> u32 {
        let Ok(mut entity_mut) = world.get_entity_mut(entity) else {
            return 0;
        };
        let Some(mut inventory) = entity_mut.get_mut::<Inventory>() else {
            return 0;
        };
        let tid = template_id.to_string();
        inventory.remove_item(&tid, quantity, weight_per_unit)
    }

    /// 装备物品到指定槽位。如果槽位已有装备，返回旧物品。
    pub fn equip_to_slot(
        world: &mut World,
        entity: Entity,
        slot: EquipSlot,
        item: ItemInstance,
    ) -> Option<ItemInstance> {
        let Ok(mut entity_mut) = world.get_entity_mut(entity) else {
            return None;
        };
        let Some(mut slots) = entity_mut.get_mut::<EquipmentSlots>() else {
            return None;
        };
        slots.equip(slot, item)
    }

    /// 从指定槽位卸下装备。返回被卸下的物品。
    pub fn unequip_from_slot(
        world: &mut World,
        entity: Entity,
        slot: EquipSlot,
    ) -> Option<ItemInstance> {
        let Ok(mut entity_mut) = world.get_entity_mut(entity) else {
            return None;
        };
        let Some(mut slots) = entity_mut.get_mut::<EquipmentSlots>() else {
            return None;
        };
        slots.unequip(slot)
    }

    /// 在实体的背包和装备间转移物品（从背包装备到槽位）。
    ///
    /// 成功则移除背包中的一件物品并装备；如果槽位有旧物，旧物放回背包。
    pub fn equip_from_inventory(
        world: &mut World,
        entity: Entity,
        template_id: &str,
        slot: EquipSlot,
        item_weight: f32,
    ) {
        let removed = Self::remove_item_from_inventory(world, entity, template_id, 1, item_weight);
        if removed == 0 {
            return;
        }
        let item = ItemInstance::new(template_id);
        if let Some(old_item) = Self::equip_to_slot(world, entity, slot, item) {
            Self::add_item_to_inventory(world, entity, old_item, item_weight);
        }
    }
}
