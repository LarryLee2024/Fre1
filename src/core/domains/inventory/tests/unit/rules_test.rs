//! 背包/物品业务规则 — 纯函数测试
//!
//! 覆盖 rules.rs 中所有规则函数在常规/边界/违规场景下的行为。
//! 规则依据：inventory_domain.md §3 不变量, §5 业务规则

use crate::core::domains::inventory::rules::rules::{
    DEFAULT_MAX_SLOTS, DEFAULT_MAX_STACK, can_stack, check_slot_compatibility, has_inventory_space,
    max_weight_from_strength, rarity_price_multiplier, stackable_amount, would_exceed_weight,
};

// ─── can_stack ─────────────────────────────────────────────────────

#[test]
fn can_stack_when_below_max() {
    assert!(can_stack(50, DEFAULT_MAX_STACK));
}

#[test]
fn can_stack_when_at_max() {
    assert!(!can_stack(DEFAULT_MAX_STACK, DEFAULT_MAX_STACK));
}

#[test]
fn can_stack_when_empty() {
    assert!(can_stack(0, DEFAULT_MAX_STACK));
}

#[test]
fn can_stack_custom_max() {
    assert!(can_stack(5, 10));
    assert!(!can_stack(10, 10));
}

// ─── stackable_amount ──────────────────────────────────────────────

#[test]
fn stackable_amount_exact_space() {
    assert_eq!(stackable_amount(50, 99, 49), 49);
}

#[test]
fn stackable_amount_more_than_space() {
    assert_eq!(stackable_amount(90, 99, 50), 9); // space=9
}

#[test]
fn stackable_amount_less_than_space() {
    assert_eq!(stackable_amount(10, 99, 5), 5);
}

#[test]
fn stackable_amount_at_max() {
    assert_eq!(stackable_amount(99, 99, 10), 0);
}

#[test]
fn stackable_amount_zero_space() {
    assert_eq!(stackable_amount(99, 99, 1), 0);
}

// ─── max_weight_from_strength ──────────────────────────────────────

#[test]
fn max_weight_from_strength_10() {
    assert_eq!(max_weight_from_strength(10), 150.0); // 10 × 15
}

#[test]
fn max_weight_from_strength_0() {
    assert_eq!(max_weight_from_strength(0), 0.0);
}

#[test]
fn max_weight_from_strength_18() {
    assert_eq!(max_weight_from_strength(18), 270.0);
}

#[test]
fn max_weight_from_strength_negative() {
    assert_eq!(max_weight_from_strength(-5), -75.0);
}

// ─── would_exceed_weight ───────────────────────────────────────────

#[test]
fn would_exceed_weight_within_limit() {
    assert!(!would_exceed_weight(100.0, 150.0, 49.0));
}

#[test]
fn would_exceed_weight_exactly_at_limit() {
    // 100 + 50 = 150, 不超出（= 不超出）
    assert!(!would_exceed_weight(100.0, 150.0, 50.0));
}

#[test]
fn would_exceed_weight_exceeds() {
    assert!(would_exceed_weight(100.0, 150.0, 51.0));
}

#[test]
fn would_exceed_weight_already_over() {
    assert!(would_exceed_weight(200.0, 150.0, 0.0));
}

// ─── check_slot_compatibility ──────────────────────────────────────

#[test]
fn check_slot_compatibility_single_handed_ok() {
    assert!(check_slot_compatibility(false, true).is_ok());
    assert!(check_slot_compatibility(false, false).is_ok());
}

#[test]
fn check_slot_compatibility_two_handed_offhand_free() {
    assert!(check_slot_compatibility(true, true).is_ok());
}

#[test]
fn check_slot_compatibility_two_handed_offhand_occupied() {
    let result = check_slot_compatibility(true, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("双手武器"));
}

// ─── rarity_price_multiplier ───────────────────────────────────────

#[test]
fn rarity_price_multiplier_common() {
    assert_eq!(rarity_price_multiplier(0), 1.0);
}

#[test]
fn rarity_price_multiplier_legendary() {
    assert_eq!(rarity_price_multiplier(4), 50.0);
}

#[test]
fn rarity_price_multiplier_out_of_range_defaults() {
    assert_eq!(rarity_price_multiplier(5), 1.0);
}

// ─── has_inventory_space ───────────────────────────────────────────

#[test]
fn has_inventory_space_can_stack() {
    assert!(has_inventory_space(20, DEFAULT_MAX_SLOTS, true));
}

#[test]
fn has_inventory_space_no_stack_with_space() {
    assert!(has_inventory_space(10, DEFAULT_MAX_SLOTS, false));
}

#[test]
fn has_inventory_space_no_stack_full() {
    assert!(!has_inventory_space(20, DEFAULT_MAX_SLOTS, false));
}

#[test]
fn has_inventory_space_no_stack_overflow() {
    assert!(!has_inventory_space(25, DEFAULT_MAX_SLOTS, false));
}
