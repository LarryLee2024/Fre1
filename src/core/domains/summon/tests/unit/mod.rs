//! Summon Domain — 单元测试
//!
//! 验证规则纯函数和组件行为。

use crate::core::domains::summon::components::{
    GridSize, SummonAIMode, SummonBond, SummonCost, SummonSlotManager, SummonTemplateDef,
};
use crate::core::domains::summon::rules::{
    can_create_concentration_summon, can_create_summon, can_summon_from_summon,
    has_free_summon_slot, is_caster_alive, is_position_valid,
    should_expire_on_concentration_broken,
};
use bevy::prelude::Entity;

// ============================================================================
// Position Validation
// ============================================================================

#[test]
fn valid_position_passable_and_unoccupied() {
    assert!(is_position_valid(0, 0, true, false));
}

#[test]
fn invalid_position_occupied() {
    assert!(!is_position_valid(0, 0, true, true));
}

#[test]
fn invalid_position_impassable() {
    assert!(!is_position_valid(0, 0, false, false));
}

// ============================================================================
// Concentration
// ============================================================================

#[test]
fn can_create_concentration_when_not_concentrating() {
    assert!(can_create_concentration_summon(false));
}

#[test]
fn cannot_create_concentration_when_already_concentrating() {
    assert!(!can_create_concentration_summon(true));
}

// ============================================================================
// Summon Slot
// ============================================================================

#[test]
fn fresh_manager_has_free_slot() {
    let manager = SummonSlotManager::new(3);
    assert!(has_free_summon_slot(&manager));
}

#[test]
fn full_manager_no_free_slot() {
    let mut manager = SummonSlotManager::new(1);
    manager.active_summons.push(Entity::PLACEHOLDER);
    assert!(!has_free_summon_slot(&manager));
}

// ============================================================================
// Caster Alive
// ============================================================================

#[test]
fn caster_alive_returns_true() {
    assert!(is_caster_alive(true));
}

#[test]
fn caster_dead_returns_false() {
    assert!(!is_caster_alive(false));
}

// ============================================================================
// Nested Summon
// ============================================================================

#[test]
fn summon_from_non_summoner_allowed() {
    assert!(can_summon_from_summon(None, true));
}

#[test]
fn summon_from_summon_is_forbidden() {
    let bond = SummonBond {
        caster: Entity::PLACEHOLDER,
        template_id: "sum_test".into(),
        ai_mode: SummonAIMode::Autonomous,
        summoned_at: 0.0,
    };
    assert!(!can_summon_from_summon(Some(&bond), false));
}

// ============================================================================
// Expire Rules
// ============================================================================

#[test]
fn concentration_summon_expires_on_concentration_broken() {
    let template = make_test_template(true);
    assert!(should_expire_on_concentration_broken(&template));
}

#[test]
fn non_concentration_summon_does_not_expire() {
    let template = make_test_template(false);
    assert!(!should_expire_on_concentration_broken(&template));
}

// ============================================================================
// Full Creation Check
// ============================================================================

#[test]
fn create_summon_all_conditions_met() {
    let template = make_test_template(true);
    let manager = SummonSlotManager::new(3);

    let result = can_create_summon(
        &template, &manager, true,  // caster alive
        false, // not concentrating
        true,  // position passable
        false, // position not occupied
        None,  // not a summon itself
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn create_summon_fails_if_caster_dead() {
    let template = make_test_template(false);
    let manager = SummonSlotManager::new(3);

    let result = can_create_summon(&template, &manager, false, true, true, false, None, false);
    assert!(result.is_err());
}

// ============================================================================
// Helpers
// ============================================================================

fn make_test_template(requires_concentration: bool) -> SummonTemplateDef {
    SummonTemplateDef {
        id: "sum_test".into(),
        name_key: "summon.test.name".into(),
        base_attributes: vec![],
        tags: vec![],
        abilities: vec![],
        modifiers: vec![],
        grid_size: GridSize::Medium,
        default_ai_mode: SummonAIMode::Autonomous,
        summon_cost: SummonCost {
            ability_id: None,
            spell_level: Some(3),
            requires_concentration,
        },
    }
}
