//! Reaction Domain — 单元测试
//!
//! 验证规则纯函数（rules.rs）。
//! 不启动 App，不加载 Plugin，不依赖资源。

use crate::core::domains::reaction::components::{
    CounterspellVerdict, ReactionState, ReactionTrigger, ReactionType,
};
use crate::core::domains::reaction::rules::{
    apply_shield_ac, calc_priority, can_opportunity_attack, can_react, can_trigger_on_turn,
    is_adjacent, resolve_counterspell, resolve_counterspell_check, resolve_opportunity_attack_hit,
};
use bevy::prelude::Entity;

// ============================================================================
// can_react
// ============================================================================

#[test]
fn fresh_state_can_react() {
    let state = ReactionState::new();
    assert!(can_react(&state));
}

#[test]
fn used_state_cannot_react() {
    let mut state = ReactionState::new();
    state.used = true;
    assert!(!can_react(&state));
}

#[test]
fn extra_reaction_available() {
    let mut state = ReactionState::new();
    state.used = true;
    state.extra_reactions = 1;
    assert!(can_react(&state));
}

#[test]
fn extra_reaction_exhausted() {
    let mut state = ReactionState::new();
    state.used = true;
    state.extra_reactions = 1;
    state.extra_used = 1;
    assert!(!can_react(&state));
}

// ============================================================================
// can_trigger_on_turn
// ============================================================================

#[test]
fn not_own_turn_allows_any_reaction() {
    assert!(can_trigger_on_turn(false, &ReactionType::OpportunityAttack));
}

#[test]
fn shield_allowed_on_own_turn() {
    assert!(can_trigger_on_turn(true, &ReactionType::Shield));
}

#[test]
fn offensive_reactions_blocked_on_own_turn() {
    assert!(!can_trigger_on_turn(true, &ReactionType::OpportunityAttack));
    assert!(!can_trigger_on_turn(true, &ReactionType::Counterspell));
}

// ============================================================================
// calc_priority
// ============================================================================

#[test]
fn shield_higher_priority_than_guardian() {
    let shield_p = calc_priority(&ReactionType::Shield, 10, 1000);
    let guardian_p = calc_priority(&ReactionType::Guardian, 10, 1000);
    assert!(shield_p > guardian_p);
}

#[test]
fn defense_over_offense_regardless_of_initiative() {
    let shield_p = calc_priority(&ReactionType::Shield, 0, 0);
    let oa_p = calc_priority(&ReactionType::OpportunityAttack, 20, 0);
    assert!(shield_p > oa_p);
}

#[test]
fn initiative_breaks_ties() {
    let high = calc_priority(&ReactionType::OpportunityAttack, 30, 0);
    let low = calc_priority(&ReactionType::OpportunityAttack, 5, 0);
    assert!(high > low);
}

// ============================================================================
// can_opportunity_attack
// ============================================================================

#[test]
fn valid_leave_threat_range_triggers() {
    let trigger = ReactionTrigger::LeaveThreatRange {
        mover: Entity::PLACEHOLDER,
        to_x: 5,
        to_y: 5,
    };
    assert!(can_opportunity_attack(&trigger, false));
}

#[test]
fn forced_movement_does_not_trigger() {
    let trigger = ReactionTrigger::LeaveThreatRange {
        mover: Entity::PLACEHOLDER,
        to_x: 5,
        to_y: 5,
    };
    assert!(!can_opportunity_attack(&trigger, true));
}

#[test]
fn wrong_trigger_type_returns_false() {
    let trigger = ReactionTrigger::EnemySpellCast {
        caster: Entity::PLACEHOLDER,
        spell_id: "spl_000001".into(),
    };
    assert!(!can_opportunity_attack(&trigger, false));
}

// ============================================================================
// resolve_opportunity_attack_hit
// ============================================================================

#[test]
fn natural_20_is_critical_hit() {
    let (hit, crit) = resolve_opportunity_attack_hit(5, 15, 20);
    assert!(hit);
    assert!(crit);
}

#[test]
fn roll_exceeds_ac_hits() {
    let (hit, crit) = resolve_opportunity_attack_hit(5, 15, 12);
    assert!(hit);
    assert!(!crit);
}

#[test]
fn roll_below_ac_misses() {
    let (hit, _crit) = resolve_opportunity_attack_hit(3, 18, 10);
    assert!(!hit);
}

#[test]
fn natural_1_always_misses() {
    let (hit, _crit) = resolve_opportunity_attack_hit(10, 5, 1);
    assert!(!hit);
}

// ============================================================================
// resolve_counterspell
// ============================================================================

#[test]
fn higher_level_counter_auto_succeeds() {
    let result = resolve_counterspell(3, 5);
    assert_eq!(result, CounterspellVerdict::AutoSuccess);
}

#[test]
fn equal_level_counter_auto_succeeds() {
    let result = resolve_counterspell(3, 3);
    assert_eq!(result, CounterspellVerdict::AutoSuccess);
}

#[test]
fn lower_level_counter_requires_check() {
    let result = resolve_counterspell(5, 3);
    match result {
        CounterspellVerdict::CheckRequired { dc, .. } => {
            assert_eq!(dc, 12);
        }
        _ => panic!("Expected CheckRequired"),
    }
}

// ============================================================================
// resolve_counterspell_check
// ============================================================================

#[test]
fn natural_20_passes_counterspell_check() {
    assert!(resolve_counterspell_check(20, 20, 0));
}

#[test]
fn natural_1_fails_counterspell_check() {
    assert!(!resolve_counterspell_check(10, 1, 10));
}

#[test]
fn total_meets_dc_passes() {
    assert!(resolve_counterspell_check(15, 12, 5));
}

#[test]
fn total_below_dc_fails() {
    assert!(!resolve_counterspell_check(15, 8, 3));
}

// ============================================================================
// is_adjacent
// ============================================================================

#[test]
fn orthogonal_positions_are_adjacent() {
    assert!(is_adjacent(5, 5, 5, 6));
    assert!(is_adjacent(5, 5, 5, 4));
    assert!(is_adjacent(5, 5, 6, 5));
    assert!(is_adjacent(5, 5, 4, 5));
}

#[test]
fn diagonal_positions_are_adjacent() {
    assert!(is_adjacent(5, 5, 6, 6));
}

#[test]
fn positions_two_away_are_not_adjacent() {
    assert!(!is_adjacent(5, 5, 7, 5));
    assert!(!is_adjacent(5, 5, 5, 7));
}

#[test]
fn same_position_is_not_adjacent() {
    assert!(!is_adjacent(5, 5, 5, 5));
}

// ============================================================================
// apply_shield_ac
// ============================================================================

#[test]
fn shield_blocks_attack() {
    let (new_ac, still_hit) = apply_shield_ac(15, 17, 5);
    assert_eq!(new_ac, 20);
    assert!(!still_hit);
}

#[test]
fn attack_still_hits_if_high_enough() {
    let (new_ac, still_hit) = apply_shield_ac(15, 22, 5);
    assert_eq!(new_ac, 20);
    assert!(still_hit);
}

#[test]
fn custom_shield_bonus() {
    let (new_ac, _still_hit) = apply_shield_ac(12, 15, 2);
    assert_eq!(new_ac, 14);
}
