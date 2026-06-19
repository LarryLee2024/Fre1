//! Crafting Domain — 制作/锻造集成测试
//!
//! 验证 CraftingPlugin 注册后的 Observer 链路：
//! - 附魔添加：on_apply_enchantment → EnchantmentSlot 更新
//! - 装备升级：on_upgrade_item → UpgradeLevel 递增

use bevy::prelude::*;

use crate::core::domains::crafting::components::{
    EnchantmentDef, EnchantmentSlot, EnchantmentSlotType, UpgradeLevel,
};
use crate::core::domains::crafting::events::{EnchantmentApplied, ItemUpgraded};
use crate::core::domains::crafting::plugin::CraftingPlugin;
use crate::core::domains::crafting::resources::EnchantmentDefRegistry;

// ─── 辅助函数 ──────────────────────────────────────────────────────

/// 创建测试 App 并注册 CraftingPlugin + 必需的 EnchantmentDefRegistry。
fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(CraftingPlugin);

    // 注册测试用附魔定义（on_apply_enchantment observer 需要 Registry 存在并包含对应 ID）
    let mut registry = EnchantmentDefRegistry::new();
    for id in &["ench_flame", "ench_frost", "ench_shock"] {
        registry.register(EnchantmentDef {
            id: id.to_string(),
            name_key: format!("enchantment.{}.name", id).into(),
            modifier_id: format!("mod_{}", id),
            exclusive_group: None,
            slot_type: EnchantmentSlotType::Weapon { max_slots: 3 },
        });
    }
    app.insert_resource(registry);
    app
}

fn spawn_enchantable_item(world: &mut World, max_slots: u32) -> Entity {
    world
        .spawn(EnchantmentSlot {
            max_slots,
            active_enchants: Vec::new(),
        })
        .id()
}

fn spawn_upgradable_item(world: &mut World, max: u32, current: u32) -> Entity {
    world
        .spawn(UpgradeLevel {
            current,
            max,
            level_modifiers: Vec::new(),
        })
        .id()
}

// ─── 附魔 ──────────────────────────────────────────────────────────

#[test]
fn apply_enchantment_adds_to_active_slot() {
    let mut app = setup_app();

    let item = spawn_enchantable_item(app.world_mut(), 3);
    app.world_mut().flush();

    app.world_mut().trigger(EnchantmentApplied {
        entity: item,
        equipment_item: "sword".into(),
        old_enchantment: None,
        new_enchantment: "ench_flame".into(),
    });
    app.world_mut().flush();

    let slot = app.world_mut().get::<EnchantmentSlot>(item).unwrap();
    assert!(
        slot.active_enchants.contains(&"ench_flame".into()),
        "附魔应被添加到活跃附魔列表"
    );
    assert_eq!(slot.active_enchants.len(), 1, "应有一个活跃附魔");
}

#[test]
fn multiple_enchantments_added_sequentially() {
    let mut app = setup_app();

    let item = spawn_enchantable_item(app.world_mut(), 3);
    app.world_mut().flush();

    for enchant in &["ench_flame", "ench_frost", "ench_shock"] {
        app.world_mut().trigger(EnchantmentApplied {
            entity: item,
            equipment_item: "sword".into(),
            old_enchantment: None,
            new_enchantment: (*enchant).into(),
        });
    }
    app.world_mut().flush();

    let slot = app.world_mut().get::<EnchantmentSlot>(item).unwrap();
    assert_eq!(slot.active_enchants.len(), 3, "可添加最多 3 个附魔");
}

#[test]
fn enchantment_slot_full_emits_failure() {
    let mut app = setup_app();

    let item = spawn_enchantable_item(app.world_mut(), 1);
    app.world_mut().flush();

    // 填满唯一位
    app.world_mut().trigger(EnchantmentApplied {
        entity: item,
        equipment_item: "sword".into(),
        old_enchantment: None,
        new_enchantment: "ench_flame".into(),
    });
    app.world_mut().flush();

    // 第二个触发 should emit CraftingFailed
    // 由于 CraftingFailed 是 commands.trigger() 发出的，需 flush 后检查状态
    app.world_mut().trigger(EnchantmentApplied {
        entity: item,
        equipment_item: "sword".into(),
        old_enchantment: None,
        new_enchantment: "ench_frost".into(),
    });
    app.world_mut().flush();

    let slot = app.world_mut().get::<EnchantmentSlot>(item).unwrap();
    assert_eq!(slot.active_enchants.len(), 1, "槽位满后不应再添加附魔");
    assert!(
        !slot.active_enchants.contains(&"ench_frost".into()),
        "被拒绝的附魔不应出现在列表中"
    );
}

// ─── 升级 ──────────────────────────────────────────────────────────

#[test]
fn upgrade_item_increments_level() {
    let mut app = App::new();
    app.add_plugins(CraftingPlugin);

    let item = spawn_upgradable_item(app.world_mut(), 5, 0);
    app.world_mut().flush();

    app.world_mut().trigger(ItemUpgraded {
        entity: item,
        equipment_item: "sword_iron".into(),
        old_level: 0,
        new_level: 1,
    });
    app.world_mut().flush();

    let level = app.world_mut().get::<UpgradeLevel>(item).unwrap();
    assert_eq!(level.current, 1, "升级后 current 应从 0 → 1");
}

#[test]
fn upgrade_at_max_level_rejected() {
    let mut app = App::new();
    app.add_plugins(CraftingPlugin);

    let item = spawn_upgradable_item(app.world_mut(), 3, 3); // 已满
    app.world_mut().flush();

    app.world_mut().trigger(ItemUpgraded {
        entity: item,
        equipment_item: "sword_iron".into(),
        old_level: 3,
        new_level: 3,
    });
    app.world_mut().flush();

    let level = app.world_mut().get::<UpgradeLevel>(item).unwrap();
    assert_eq!(level.current, 3, "已达上限时升级不应增加等级");
}

#[test]
fn sequential_upgrades_increment_step_by_step() {
    let mut app = App::new();
    app.add_plugins(CraftingPlugin);

    let item = spawn_upgradable_item(app.world_mut(), 5, 0);
    app.world_mut().flush();

    // 连续升级 3 次
    for i in 0..3u32 {
        app.world_mut().trigger(ItemUpgraded {
            entity: item,
            equipment_item: "sword_iron".into(),
            old_level: i,
            new_level: i + 1,
        });
    }
    app.world_mut().flush();

    let level = app.world_mut().get::<UpgradeLevel>(item).unwrap();
    assert_eq!(level.current, 3, "连续 3 次升级应从 0 → 3");
}
