//! Crafting Domain — 单元测试
//!
//! 验证规则纯函数和组件行为。

use crate::core::domains::crafting::components::{
    CraftType, CraftingStation, EnchantmentSlot, MaterialCost, RecipeDef, SkillRequirement,
    UpgradeLevel,
};
use crate::core::domains::crafting::rules::{
    check_materials_available, check_station_match, check_upgrade_limit, has_free_enchantment_slot,
    perform_skill_check,
};

// ============================================================================
// Recipe Checking
// ============================================================================

#[test]
fn station_match_succeeds() {
    let recipe = make_test_recipe();
    assert!(check_station_match(&recipe, CraftingStation::Forge));
}

#[test]
fn station_mismatch_fails() {
    let recipe = make_test_recipe();
    assert!(!check_station_match(&recipe, CraftingStation::AlchemyLab));
}

#[test]
fn materials_available_succeeds() {
    let recipe = make_test_recipe();
    let inventory = |id: &str| match id {
        "itm_iron_ingot" => 5,
        "itm_wood" => 3,
        _ => 0,
    };
    assert!(check_materials_available(&recipe, &inventory).is_ok());
}

#[test]
fn materials_missing_fails() {
    let recipe = make_test_recipe();
    let inventory = |_id: &str| 0;
    assert!(check_materials_available(&recipe, &inventory).is_err());
}

// ============================================================================
// Enchantment Slots
// ============================================================================

#[test]
fn empty_slot_has_free_space() {
    let slot = EnchantmentSlot {
        max_slots: 3,
        active_enchants: vec![],
    };
    assert!(has_free_enchantment_slot(&slot));
}

#[test]
fn full_slot_no_free_space() {
    let slot = EnchantmentSlot {
        max_slots: 2,
        active_enchants: vec!["enc_fire".into(), "enc_ice".into()],
    };
    assert!(!has_free_enchantment_slot(&slot));
}

// ============================================================================
// Upgrade Level
// ============================================================================

#[test]
fn fresh_upgrade_can_upgrade() {
    let level = UpgradeLevel::new(3);
    assert!(check_upgrade_limit(&level));
}

#[test]
fn maxed_upgrade_cannot_upgrade() {
    let level = UpgradeLevel {
        current: 3,
        max: 3,
        level_modifiers: vec![],
    };
    assert!(!check_upgrade_limit(&level));
}

// ============================================================================
// Skill Check
// ============================================================================

#[test]
fn skill_check_with_bonus_succeeds() {
    assert!(perform_skill_check(5, 13, 20));
}

#[test]
fn skill_check_without_bonus_fails() {
    assert!(!perform_skill_check(0, 10, 20));
}

// ============================================================================
// Helpers
// ============================================================================

fn make_test_recipe() -> RecipeDef {
    RecipeDef {
        id: "rcp_test".into(),
        name_key: "recipe.test.name".into(),
        station: CraftingStation::Forge,
        skill_requirement: Some(SkillRequirement {
            skill_id: "smithing".into(),
            dc: 13,
        }),
        materials: vec![
            MaterialCost {
                item_id: "itm_iron_ingot".into(),
                quantity: 3,
            },
            MaterialCost {
                item_id: "itm_wood".into(),
                quantity: 1,
            },
        ],
        output: crate::core::domains::crafting::components::CraftOutput {
            item_id: "itm_longsword_1".into(),
            quantity: 1,
            enchantment_slots: 2,
        },
        craft_time: 2,
        craft_type: CraftType::Smithing,
    }
}
