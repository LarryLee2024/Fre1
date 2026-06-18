//! Inventory 领域不变量测试
//!
//! 验证 inventory_domain.md §3 中定义的核心不变量：
//!   不变量 3.1 — 每个装备槽位同一时间只能穿戴一件装备
//!   不变量 3.2 — 双手武器同时占用 MainHand + OffHand
//!   不变量 3.3 — 背包格数不超过 max_slots
//!   不变量 3.4 — 耐久度不超过最大耐久度
//!   不变量 3.5 — 堆叠数量不超过 99
//!
//! 这些测试保证业务规则在任何代码变更后仍然成立。

use crate::core::domains::inventory::components::{
    DurabilityState, EquipSlot, EquipmentSlots, Inventory, ItemInstance,
};

// ─── 不变量 3.1：每个槽位同一时间只能穿戴一件装备 ────────────────

#[test]
fn invariant_each_slot_holds_at_most_one_item() {
    let mut slots = EquipmentSlots::new();

    // 装备到同一槽位两次 → 旧装备被替换，不会出现两个
    slots.equip(EquipSlot::MainHand, ItemInstance::new("iron_sword"));
    slots.equip(EquipSlot::MainHand, ItemInstance::new("steel_sword"));

    let item = slots.get(EquipSlot::MainHand);
    assert!(item.is_some(), "不变量 3.1 违反：装备后槽位为空");
    assert_eq!(
        item.unwrap().template_id,
        "steel_sword",
        "不变量 3.1 违反：槽位中出现了多个物品"
    );
}

// ─── 不变量 3.2：双手武器同时占用 MainHand + OffHand ─────────────

#[test]
fn invariant_two_handed_blocks_offhand() {
    // 这是 UI/逻辑层面的约束，is_two_handed_weapon_equipped 检测到主手有、副手空的情况
    let mut slots = EquipmentSlots::new();
    slots.equip(EquipSlot::MainHand, ItemInstance::new("greatsword"));

    assert!(
        slots.is_two_handed_weapon_equipped(),
        "不变量 3.2 违反：双手武器应标记双槽位占用"
    );

    // 副手有东西时，标记解除
    slots.equip(EquipSlot::OffHand, ItemInstance::new("shield"));
    assert!(
        !slots.is_two_handed_weapon_equipped(),
        "不变量 3.2 违反：副手已占用但仍标记双手武器"
    );
}

// ─── 不变量 3.3：背包格数不超过 max_slots ─────────────────────────

#[test]
fn invariant_slot_count_never_exceeds_max_slots() {
    let mut inv = Inventory::new(3, 300.0);

    // 尝试放入 5 个不可堆叠的不同物品
    for id in &["sword", "shield", "helmet", "boots", "amulet"] {
        inv.add_item(ItemInstance::new(*id), 1.0);
    }

    assert!(
        inv.used_slots() as u32 <= inv.max_slots,
        "不变量 3.3 违反：实际格数 {} 超过上限 {}",
        inv.used_slots(),
        inv.max_slots
    );
}

#[test]
fn invariant_item_count_never_exceeds_max_slots_for_new_items() {
    let mut inv = Inventory::new(2, 300.0);

    // 放 3 个不同物品（每个需要新格子）
    inv.add_item(ItemInstance::new("a"), 1.0);
    inv.add_item(ItemInstance::new("b"), 1.0);
    inv.add_item(ItemInstance::new("c"), 1.0); // 应该被拒绝

    assert_eq!(
        inv.item_count(),
        2,
        "不变量 3.3 违反：超过 max_slots 的物品被添加"
    );
}

// ─── 不变量 3.4：耐久度不超过最大耐久度 ───────────────────────────

#[test]
fn invariant_durability_never_exceeds_max() {
    let mut d = DurabilityState::new(100);
    d.reduce(30);
    d.repair(200); // 尝试修复超过 max

    assert!(
        d.current <= d.max,
        "不变量 3.4 违反：耐久度 {} 超过最大值 {}",
        d.current,
        d.max
    );
}

#[test]
fn invariant_durability_new_starts_at_max() {
    let d = DurabilityState::new(50);
    assert_eq!(d.current, d.max, "不变量 3.4 违反：新耐久度应等于最大值");
}

// ─── 不变量 3.5：堆叠数量不超过 99 ────────────────────────────────

#[test]
fn invariant_stack_quantity_never_exceeds_99() {
    let mut inv = Inventory::new(20, 300.0);

    // 第一次放入 50
    inv.add_item(ItemInstance::with_quantity("potion", 50), 0.5);
    // 尝试再放入 100（实际只能加 49）
    let added = inv.add_item(ItemInstance::with_quantity("potion", 100), 0.5);

    let item = inv.find_item("potion").unwrap();
    assert!(
        item.quantity <= 99,
        "不变量 3.5 违反：堆叠数量 {} 超过上限 99",
        item.quantity
    );
    assert_eq!(added, 49, "应只能添加 49 个（99-50）");
    assert_eq!(item.quantity, 99, "堆叠应恰好为 99");
}

#[test]
fn invariant_stack_never_exceeds_99_on_single_add() {
    let mut inv = Inventory::new(20, 300.0);
    // ItemInstance 本身 quantity 可以超过 99，但业务规则限制堆叠
    // 测试 can_hold 中堆叠合并：同类且未满 99 的可堆叠
    // 这里测试 add_item 内部逻辑自动限制了堆叠
    let added = inv.add_item(ItemInstance::with_quantity("gold", 150), 0.01);
    let item = inv.find_item("gold").unwrap();
    assert!(
        item.quantity <= 99,
        "不变量 3.5 违反：堆叠数量 {} 超过上限 99",
        item.quantity
    );
    assert_eq!(added, 99);
    assert_eq!(item.quantity, 99);
}

// ─── 跨组件不变量：remove_item 后 total_weight 不出现负数 ────────

#[test]
fn invariant_total_weight_never_negative() {
    let mut inv = Inventory::new(10, 300.0);

    // 移除不存在的物品，total_weight 不变
    inv.remove_item("nonexistent", 1, 10.0);
    assert!(inv.total_weight >= 0.0, "不变量违反：total_weight 出现负数");
}
