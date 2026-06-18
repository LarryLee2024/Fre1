//! 成长业务规则 — 纯函数测试
//!
//! 覆盖 rules.rs 中所有规则函数在签约/边界/违规场景下的行为。
//! 规则依据：progression_domain.md §3 不变量, §5 业务规则

use crate::core::domains::progression::rules::rules::{
    can_level_up, check_asi_attribute_increase, check_multiclass_prerequisites,
    check_talent_prerequisites, max_achievable_level, proficiency_for_total_level,
    validate_xp_gain, xp_after_level_up,
};

// ─── can_level_up ──────────────────────────────────────────────────

#[test]
fn can_level_up_lv1_enough_xp() {
    assert!(can_level_up(1, 300));
}

#[test]
fn can_level_up_lv1_insufficient_xp() {
    assert!(!can_level_up(1, 299));
}

#[test]
fn can_level_up_max_level_always_false() {
    assert!(!can_level_up(20, u64::MAX));
}

#[test]
fn can_level_up_exact_threshold() {
    // 到 3 级需要累计 900 XP
    assert!(can_level_up(2, 900));
    assert!(!can_level_up(2, 899));
}

// ─── xp_after_level_up ─────────────────────────────────────────────

#[test]
fn xp_after_level_up_exact_consumption() {
    // lv1→lv2 需要 300 XP, 恰好 300 → 剩余 0
    assert_eq!(xp_after_level_up(300, 1), 0);
}

#[test]
fn xp_after_level_up_with_overflow() {
    // 有 500 XP, lv1, 消耗 300 升到 lv2, 剩余 200
    assert_eq!(xp_after_level_up(500, 1), 200);
}

#[test]
fn xp_after_level_up_saturating_sub() {
    // 经验不足时返回 0 (saturating_sub)
    assert_eq!(xp_after_level_up(100, 1), 0);
}

// ─── max_achievable_level ──────────────────────────────────────────

#[test]
fn max_achievable_level_no_level_up() {
    assert_eq!(max_achievable_level(0, 1), 1);
}

#[test]
fn max_achievable_level_one_level_up() {
    // 300 XP at lv1 → lv2
    assert_eq!(max_achievable_level(300, 1), 2);
}

#[test]
fn max_achievable_level_multiple_levels() {
    // 从 lv1 开始, 10,000 XP → lv5 (累计 6500), 剩余够升 lv6 吗? 不行, lv5→lv6 需要 14000−6500=7500
    // 10,000 够到 lv5, 不够 lv6
    assert_eq!(max_achievable_level(10_000, 1), 5);
}

#[test]
fn max_achievable_level_all_the_way() {
    // 足够 XP 直接到满级
    assert_eq!(max_achievable_level(u64::MAX, 1), 20);
}

#[test]
fn max_achievable_level_already_max() {
    assert_eq!(max_achievable_level(0, 20), 20);
}

// ─── proficiency_for_total_level ───────────────────────────────────

#[test]
fn proficiency_for_total_level_delegates_correctly() {
    assert_eq!(proficiency_for_total_level(1), 2);
    assert_eq!(proficiency_for_total_level(5), 3);
    assert_eq!(proficiency_for_total_level(17), 6);
}

// ─── check_talent_prerequisites ────────────────────────────────────

#[test]
fn check_talent_prerequisites_ok() {
    let result = check_talent_prerequisites(3, 5, &[], &[]);
    assert!(result.is_ok());
}

#[test]
fn check_talent_prerequisites_level_too_low() {
    let result = check_talent_prerequisites(5, 3, &[], &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("需要等级"));
}

#[test]
fn check_talent_prerequisites_missing_prereq_talent() {
    let result = check_talent_prerequisites(3, 5, &["talent_alpha".to_string()], &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("前置天赋"));
}

#[test]
fn check_talent_prerequisites_all_prereqs_met() {
    let result = check_talent_prerequisites(
        3,
        5,
        &["talent_a".to_string(), "talent_b".to_string()],
        &["talent_a".to_string(), "talent_b".to_string()],
    );
    assert!(result.is_ok());
}

// ─── check_multiclass_prerequisites ─────────────────────────────────

#[test]
fn check_multiclass_prerequisites_ok() {
    let checks = &[("力量", 13, 15), ("敏捷", 13, 14)];
    let result = check_multiclass_prerequisites(5, checks);
    assert!(result.is_ok());
}

#[test]
fn check_multiclass_prerequisites_max_level_fails() {
    let result = check_multiclass_prerequisites(20, &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("满级"));
}

#[test]
fn check_multiclass_prerequisites_attribute_too_low() {
    let checks = &[("力量", 13, 10)];
    let result = check_multiclass_prerequisites(5, checks);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("力量"));
}

// ─── check_asi_attribute_increase ──────────────────────────────────

#[test]
fn check_asi_attribute_increase_ok() {
    assert_eq!(check_asi_attribute_increase(10, 2), Ok(12));
}

#[test]
fn check_asi_attribute_increase_exceeds_20() {
    let result = check_asi_attribute_increase(19, 2);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("20"));
}

#[test]
fn check_asi_attribute_increase_exactly_20() {
    assert_eq!(check_asi_attribute_increase(18, 2), Ok(20));
}

#[test]
fn check_asi_attribute_increase_negative_fails() {
    let result = check_asi_attribute_increase(10, -1);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("降低"));
}

// ─── validate_xp_gain ──────────────────────────────────────────────

#[test]
fn validate_xp_gain_positive_ok() {
    assert!(validate_xp_gain(100).is_ok());
}

#[test]
fn validate_xp_gain_zero_fails() {
    assert!(validate_xp_gain(0).is_err());
}
