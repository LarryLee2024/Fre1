//! 背包/物品领域 Component — 行为测试
//!
//! 覆盖 components.rs 中所有 ECS Component 的方法在常规/边界/错误场景下的行为。
//! 规则依据：inventory_domain.md §3 不变量

use crate::core::domains::inventory::components::{
    ArmorCategory, DurabilityState, EquipSlot, EquipmentSlots, Inventory, ItemInstance, ItemType,
    Rarity, WeaponCategory,
};

// ─── EquipSlot ─────────────────────────────────────────────────────

#[test]
fn equip_slot_all_returns_11_slots() {
    let slots: Vec<_> = EquipSlot::all().collect();
    assert_eq!(slots.len(), 11);
    assert!(slots.contains(&EquipSlot::MainHand));
    assert!(slots.contains(&EquipSlot::Special));
}

#[test]
fn equip_slot_name_returns_chinese() {
    use EquipSlot::*;
    assert_eq!(MainHand.name(), "主手");
    assert_eq!(OffHand.name(), "副手");
    assert_eq!(Helmet.name(), "头盔");
    assert_eq!(Armor.name(), "铠甲");
    assert_eq!(Gloves.name(), "手套");
    assert_eq!(Boots.name(), "靴子");
    assert_eq!(Cloak.name(), "披风");
    assert_eq!(Ring1.name(), "戒指1");
    assert_eq!(Ring2.name(), "戒指2");
    assert_eq!(Amulet.name(), "项链");
    assert_eq!(Special.name(), "特殊");
}

// ─── DurabilityState ───────────────────────────────────────────────

#[test]
fn durability_new_starts_full() {
    let d = DurabilityState::new(100);
    assert_eq!(d.current, 100);
    assert_eq!(d.max, 100);
    assert!(!d.is_broken);
}

#[test]
fn durability_reduce_decrements() {
    let mut d = DurabilityState::new(100);
    d.reduce(30);
    assert_eq!(d.current, 70);
    assert!(!d.is_broken);
}

#[test]
fn durability_reduce_to_zero_triggers_broken() {
    let mut d = DurabilityState::new(100);
    d.reduce(100);
    assert_eq!(d.current, 0);
    assert!(d.is_broken);
}

#[test]
fn durability_reduce_beyond_zero_saturates() {
    let mut d = DurabilityState::new(50);
    d.reduce(100);
    assert_eq!(d.current, 0);
    assert!(d.is_broken);
}

#[test]
fn durability_repair_increases() {
    let mut d = DurabilityState::new(100);
    d.reduce(50);
    d.repair(20);
    assert_eq!(d.current, 70);
    assert!(!d.is_broken);
}

#[test]
fn durability_repair_does_not_exceed_max() {
    let mut d = DurabilityState::new(100);
    d.reduce(10);
    d.repair(50);
    assert_eq!(d.current, 100); // 修复不会超过 max
}

#[test]
fn durability_repair_from_broken() {
    let mut d = DurabilityState::new(100);
    d.reduce(100);
    assert!(d.is_broken);
    d.repair(100);
    assert!(!d.is_broken);
    assert_eq!(d.current, 100);
}

// ─── ItemType / Rarity / WeaponCategory / ArmorCategory ───────────

#[test]
fn rarity_default_is_common() {
    let r: Rarity = Default::default();
    assert_eq!(r, Rarity::Common);
}

#[test]
fn item_type_weapon_holds_category() {
    let t = ItemType::Weapon(WeaponCategory::MartialMelee);
    assert!(matches!(t, ItemType::Weapon(WeaponCategory::MartialMelee)));
}

#[test]
fn item_type_armor_holds_category() {
    let t = ItemType::Armor(ArmorCategory::Heavy);
    assert!(matches!(t, ItemType::Armor(ArmorCategory::Heavy)));
}

// ─── ItemInstance ──────────────────────────────────────────────────

#[test]
fn item_instance_new_defaults_quantity_1() {
    let item = ItemInstance::new("longsword");
    assert_eq!(item.template_id, "longsword");
    assert_eq!(item.quantity, 1);
    assert!(item.durability.is_none());
    assert!(item.enchants.is_empty());
}

#[test]
fn item_instance_with_quantity() {
    let item = ItemInstance::with_quantity("arrow", 20);
    assert_eq!(item.quantity, 20);
}

#[test]
fn item_instance_is_stackable_when_no_durability() {
    let item = ItemInstance::new("gold_coin");
    assert!(item.is_stackable());
}

#[test]
fn item_instance_not_stackable_with_durability() {
    let item = ItemInstance {
        template_id: "magic_sword".to_string(),
        quantity: 1,
        durability: Some(DurabilityState::new(50)),
        enchants: vec![],
    };
    assert!(!item.is_stackable());
}

// ─── Inventory ─────────────────────────────────────────────────────

#[test]
fn inventory_new_has_zero_items() {
    let inv = Inventory::new(20, 300.0);
    assert_eq!(inv.item_count(), 0);
    assert_eq!(inv.max_slots, 20);
    assert_eq!(inv.max_weight, 300.0);
}

#[test]
fn inventory_default() {
    let inv = Inventory::default();
    assert_eq!(inv.max_slots, 20);
    assert!((inv.max_weight - 300.0).abs() < f32::EPSILON);
}

#[test]
fn inventory_used_slots_matches_item_count() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::new("sword"), 2.0);
    inv.add_item(ItemInstance::new("shield"), 5.0);
    assert_eq!(inv.used_slots(), 2);
}

#[test]
fn inventory_remaining_slots() {
    let mut inv = Inventory::new(5, 300.0);
    assert_eq!(inv.remaining_slots(), 5);
    inv.add_item(ItemInstance::new("sword"), 2.0);
    assert_eq!(inv.remaining_slots(), 4);
}

#[test]
fn inventory_has_free_slot_positive() {
    let inv = Inventory::new(5, 300.0);
    assert!(inv.has_free_slot());
}

#[test]
fn inventory_has_free_slot_full() {
    let mut inv = Inventory::new(1, 300.0);
    inv.add_item(ItemInstance::new("sword"), 2.0);
    assert!(!inv.has_free_slot());
}

#[test]
fn inventory_can_hold_with_space() {
    let inv = Inventory::new(20, 300.0);
    let item = ItemInstance::new("potion");
    assert!(inv.can_hold(&item, 1.0));
}

#[test]
fn inventory_can_hold_exceeds_weight() {
    let inv = Inventory::new(20, 10.0);
    let item = ItemInstance::new("heavy_armor");
    assert!(!inv.can_hold(&item, 20.0));
}

#[test]
fn inventory_can_hold_full_slots() {
    let mut inv = Inventory::new(1, 300.0);
    inv.add_item(ItemInstance::new("sword"), 2.0);
    // 尝试放入不同模板的物品（需要新格子）
    let potion = ItemInstance::new("potion");
    assert!(!inv.can_hold(&potion, 1.0));
}

#[test]
fn inventory_can_hold_stackable_merge() {
    let mut inv = Inventory::new(1, 300.0);
    inv.add_item(ItemInstance::with_quantity("potion", 50), 0.5);
    // 同类物品可堆叠，不需要新格子
    let more = ItemInstance::with_quantity("potion", 10);
    assert!(inv.can_hold(&more, 0.5));
}

#[test]
fn inventory_add_item_stack_merge() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("potion", 50), 0.5);
    let added = inv.add_item(ItemInstance::with_quantity("potion", 60), 0.5);
    // 只能再堆叠 49 个（99-50）
    assert_eq!(added, 49);
    assert_eq!(inv.total_weight, (50 + 49) as f32 * 0.5);
}

#[test]
fn inventory_add_item_new_slot() {
    let mut inv = Inventory::new(20, 300.0);
    let added = inv.add_item(ItemInstance::new("sword"), 3.0);
    assert_eq!(added, 1);
    assert_eq!(inv.item_count(), 1);
    assert!((inv.total_weight - 3.0).abs() < f32::EPSILON);
}

#[test]
fn inventory_add_item_no_space() {
    let mut inv = Inventory::new(0, 300.0);
    let added = inv.add_item(ItemInstance::new("sword"), 3.0);
    assert_eq!(added, 0);
}

#[test]
fn inventory_remove_item_partial() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("arrow", 20), 0.1);
    let removed = inv.remove_item("arrow", 5, 0.1);
    assert_eq!(removed, 5);
    assert_eq!(inv.find_item("arrow").unwrap().quantity, 15);
}

#[test]
fn inventory_remove_item_exact_removes_entry() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("potion", 3), 0.5);
    let removed = inv.remove_item("potion", 3, 0.5);
    assert_eq!(removed, 3);
    assert!(inv.find_item("potion").is_none());
}

#[test]
fn inventory_remove_item_more_than_available() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("arrow", 5), 0.1);
    let removed = inv.remove_item("arrow", 100, 0.1);
    assert_eq!(removed, 5);
    assert!(inv.find_item("arrow").is_none());
}

#[test]
fn inventory_remove_item_not_found() {
    let mut inv = Inventory::new(20, 300.0);
    let removed = inv.remove_item("nonexistent", 1, 1.0);
    assert_eq!(removed, 0);
}

#[test]
fn inventory_has_item_sufficient() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("potion", 5), 0.5);
    assert!(inv.has_item("potion", 3));
}

#[test]
fn inventory_has_item_insufficient() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("potion", 2), 0.5);
    assert!(!inv.has_item("potion", 3));
}

#[test]
fn inventory_has_item_not_found() {
    let inv = Inventory::new(20, 300.0);
    assert!(!inv.has_item("potion", 1));
}

#[test]
fn inventory_find_item_mut_allows_modification() {
    let mut inv = Inventory::new(20, 300.0);
    inv.add_item(ItemInstance::with_quantity("potion", 5), 0.5);
    if let Some(item) = inv.find_item_mut("potion") {
        item.quantity += 5;
    }
    assert_eq!(inv.find_item("potion").unwrap().quantity, 10);
}

// ─── EquipmentSlots ────────────────────────────────────────────────

#[test]
fn equipment_slots_new_all_empty() {
    let slots = EquipmentSlots::new();
    for slot in EquipSlot::all() {
        assert!(slots.is_slot_empty(slot), "{:?} should be empty", slot);
    }
}

#[test]
fn equipment_slots_equip_places_item() {
    let mut slots = EquipmentSlots::new();
    let sword = ItemInstance::new("longsword");
    let old = slots.equip(EquipSlot::MainHand, sword.clone());
    assert!(old.is_none());
    assert_eq!(
        slots.get(EquipSlot::MainHand).unwrap().template_id,
        "longsword"
    );
}

#[test]
fn equipment_slots_equip_replaces_existing() {
    let mut slots = EquipmentSlots::new();
    slots.equip(EquipSlot::MainHand, ItemInstance::new("iron_sword"));
    let old = slots.equip(EquipSlot::MainHand, ItemInstance::new("steel_sword"));
    assert!(old.is_some());
    assert_eq!(old.unwrap().template_id, "iron_sword");
    assert_eq!(
        slots.get(EquipSlot::MainHand).unwrap().template_id,
        "steel_sword"
    );
}

#[test]
fn equipment_slots_unequip_removes_item() {
    let mut slots = EquipmentSlots::new();
    slots.equip(EquipSlot::Helmet, ItemInstance::new("iron_helmet"));
    let removed = slots.unequip(EquipSlot::Helmet);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().template_id, "iron_helmet");
    assert!(slots.is_slot_empty(EquipSlot::Helmet));
}

#[test]
fn equipment_slots_unequip_empty() {
    let mut slots = EquipmentSlots::new();
    assert!(slots.unequip(EquipSlot::Amulet).is_none());
}

#[test]
fn equipment_slots_get_returns_correct_slot() {
    let mut slots = EquipmentSlots::new();
    slots.equip(EquipSlot::Boots, ItemInstance::new("leather_boots"));
    assert_eq!(
        slots.get(EquipSlot::Boots).unwrap().template_id,
        "leather_boots"
    );
    assert!(slots.get(EquipSlot::Helmet).is_none());
}

#[test]
fn equipment_slots_is_two_handed_weapon_equipped_false_by_default() {
    let slots = EquipmentSlots::new();
    assert!(!slots.is_two_handed_weapon_equipped());
}

#[test]
fn equipment_slots_is_two_handed_true_when_main_hand_only() {
    let mut slots = EquipmentSlots::new();
    slots.equip(EquipSlot::MainHand, ItemInstance::new("greatsword"));
    // 主手有装备，副手空 → 视为双手武器
    assert!(slots.is_two_handed_weapon_equipped());
}

#[test]
fn equipment_slots_is_two_handed_false_with_offhand() {
    let mut slots = EquipmentSlots::new();
    slots.equip(EquipSlot::MainHand, ItemInstance::new("greatsword"));
    slots.equip(EquipSlot::OffHand, ItemInstance::new("shield"));
    assert!(!slots.is_two_handed_weapon_equipped());
}

#[test]
fn equipment_slots_default_equals_new() {
    assert_eq!(EquipmentSlots::default(), EquipmentSlots::new());
}
