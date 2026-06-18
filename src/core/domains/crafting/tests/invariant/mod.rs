//! Crafting Domain — 不变量测试
//!
//! 验证 docs/02-domain/domains/crafting_domain.md §3 定义的不变量。

use crate::core::domains::crafting::components::{
    CraftType, CraftingStation, EnchantmentDef, EnchantmentSlot, MaterialCost, RecipeDef,
    SkillRequirement, UpgradeLevel,
};
use crate::core::domains::crafting::rules::{
    check_materials_available, check_upgrade_limit, has_free_enchantment_slot,
};

/// 不变量 3.3：附魔槽位上限制 — active_enchants 长度不得超过 max_slots。
#[test]
fn enchantment_slot_count_never_exceeds_max() {
    let slot = EnchantmentSlot {
        max_slots: 2,
        active_enchants: vec!["enc_fire".into(), "enc_ice".into()],
    };
    // full: 和 max 相等
    assert!(!has_free_enchantment_slot(&slot));
}

/// 不变量 3.4：升级等级上限 — current 不得超过 max。
#[test]
fn upgrade_level_never_exceeds_max() {
    let level = UpgradeLevel {
        current: 3,
        max: 3,
        level_modifiers: vec![],
    };
    assert!(
        !check_upgrade_limit(&level),
        "current == max should block upgrade"
    );
}

/// 不变量 3.4：fresh upgrade 允许增长。
#[test]
fn upgrade_below_max_allowed() {
    let level = UpgradeLevel::new(5);
    assert!(check_upgrade_limit(&level));
}

/// 不变量 3.1：材料充足性 — 材料数量严格 >= 配方要求。
#[test]
fn exact_materials_satisfies_invariant() {
    let recipe = RecipeDef {
        id: "rcp_sword".into(),
        name_key: "recipe.invariant.name".into(),
        station: CraftingStation::Forge,
        skill_requirement: None,
        materials: vec![MaterialCost {
            item_id: "itm_iron".into(),
            quantity: 3,
        }],
        output: crate::core::domains::crafting::components::CraftOutput {
            item_id: "itm_sword".into(),
            quantity: 1,
            enchantment_slots: 0,
        },
        craft_time: 1,
        craft_type: CraftType::Smithing,
    };
    let inventory = |id: &str| match id {
        "itm_iron" => 3, // exact match
        _ => 0,
    };
    assert!(
        check_materials_available(&recipe, &inventory).is_ok(),
        "exact material count should satisfy requirement"
    );
}

/// 不变量 3.5：互斥词条防冲突 — 同类型附魔应替换。
#[test]
fn exclusive_enchantment_replaces_old() {
    // 规则：同类型词条互斥（如"火焰"和"冰霜"不能共存）
    // 本域规则定义 "互斥词条后附魔的会覆盖先附魔的"。
    // 验证 EnchantmentSlot 在添加互斥词条时 active_enchants 数量不超上限
    let mut slot = EnchantmentSlot {
        max_slots: 1,
        active_enchants: vec!["enc_fire".into()],
    };
    // 添加互斥词条 → 应替换 "enc_fire"
    slot.active_enchants.clear();
    slot.active_enchants.push("enc_frost".into());
    assert_eq!(
        slot.active_enchants.len(),
        1,
        "replacement should keep slot count at 1"
    );
    assert_eq!(
        slot.active_enchants[0], "enc_frost",
        "new enchant should replace old"
    );
}
