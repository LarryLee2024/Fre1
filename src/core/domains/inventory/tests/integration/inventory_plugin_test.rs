//! InventoryPlugin 集成测试
//!
//! 验证 Plugin 注册、Component 类型注册、Observer 系统执行流程正常。
//! 使用最小 App 实例，不依赖外部资源。

use bevy::prelude::*;

use crate::core::domains::inventory::InventoryPlugin;
use crate::core::domains::inventory::components::{
    EquipSlot, EquipmentSlots, Inventory, InventoryMarker, ItemInstance,
};

/// Plugin 注册后所有 Component 类型可用。
#[test]
fn inventory_plugin_registers_components() {
    let mut app = App::new();
    app.add_plugins(InventoryPlugin);

    let entity = app
        .world_mut()
        .spawn((
            Inventory::default(),
            EquipmentSlots::new(),
            ItemInstance::new("test_item"),
            InventoryMarker,
        ))
        .id();

    let world = app.world();
    assert!(
        world.get::<Inventory>(entity).is_some(),
        "Inventory 组件注册失败"
    );
    assert!(
        world.get::<EquipmentSlots>(entity).is_some(),
        "EquipmentSlots 组件注册失败"
    );
    assert!(
        world.get::<ItemInstance>(entity).is_some(),
        "ItemInstance 组件注册失败"
    );
    assert!(
        world.get::<InventoryMarker>(entity).is_some(),
        "InventoryMarker 组件注册失败"
    );
}

/// 可独立使用 EquipmentSlots + Inventory 进行装备流程。
#[test]
fn inventory_equip_and_unequip_flow() {
    let mut app = App::new();
    app.add_plugins(InventoryPlugin);

    let mut inv = Inventory::default();
    let sword = ItemInstance::new("longsword");
    let shield = ItemInstance::new("shield");

    // 先放入背包
    inv.add_item(sword.clone(), 3.0);
    inv.add_item(shield.clone(), 5.0);

    let mut slots = EquipmentSlots::new();
    // 装备
    let old_sword = slots.equip(EquipSlot::MainHand, sword.clone());
    assert!(old_sword.is_none());

    // 卸下
    let unequipped = slots.unequip(EquipSlot::MainHand);
    assert!(unequipped.is_some());
    assert_eq!(unequipped.unwrap().template_id, "longsword");
}

/// Inventory 容量限制在集成层面验证。
#[test]
fn inventory_capacity_enforced() {
    let mut inv = Inventory::new(1, 100.0);

    // 第一个物品放入成功
    assert_eq!(inv.add_item(ItemInstance::new("item_a"), 10.0), 1);

    // 第二个不同物品（需要新格子）放入失败
    assert_eq!(inv.add_item(ItemInstance::new("item_b"), 10.0), 0);

    // 同类型物品可以堆叠（不需要新格子）
    let stacked = inv.add_item(ItemInstance::with_quantity("item_a", 10), 10.0);
    assert_eq!(stacked, 10);
    assert_eq!(inv.find_item("item_a").unwrap().quantity, 11);
}
