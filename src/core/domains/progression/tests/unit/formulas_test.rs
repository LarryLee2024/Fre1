//! 经验与成长计算公式 — 纯函数测试
//!
//! 覆盖 formulas.rs 中所有纯函数在常规/边界/极端输入下的行为。
//! 规则依据：progression_domain.md §1 经验曲线

use crate::core::domains::progression::rules::formulas::{
    MAX_LEVEL, asi_levels, cumulative_xp_for_level, is_asi_level, level_from_xp, next_asi_level,
    proficiency_bonus, xp_to_next_level,
};

// ─── xp_to_next_level ──────────────────────────────────────────────

#[test]
fn xp_to_next_level_lv1_is_300() {
    // lv1→lv2 需要 300 XP（D&D 5e 标准）
    assert_eq!(xp_to_next_level(1), 300);
}

#[test]
fn xp_to_next_level_lv19_is_50000() {
    // lv19→lv20 需要 50,000 XP（355000−305000）
    assert_eq!(xp_to_next_level(19), 50_000);
}

#[test]
fn xp_to_next_level_max_level_returns_zero() {
    assert_eq!(xp_to_next_level(MAX_LEVEL), 0);
}

#[test]
fn xp_to_next_level_above_max_returns_zero() {
    assert_eq!(xp_to_next_level(MAX_LEVEL + 5), 0);
}

// ─── cumulative_xp_for_level ───────────────────────────────────────

#[test]
fn cumulative_xp_for_level_1_is_zero() {
    assert_eq!(cumulative_xp_for_level(1), 0);
}

#[test]
fn cumulative_xp_for_level_2_is_300() {
    // 到 2 级需要累计 300 XP
    assert_eq!(cumulative_xp_for_level(2), 300);
}

#[test]
fn cumulative_xp_for_level_20_is_355000() {
    // 到 20 级需要 355,000 XP
    assert_eq!(cumulative_xp_for_level(20), 355_000);
}

#[test]
fn cumulative_xp_for_level_above_max_returns_u64_max() {
    assert_eq!(cumulative_xp_for_level(MAX_LEVEL + 1), u64::MAX);
}

#[test]
fn cumulative_xp_for_level_zero_returns_zero() {
    assert_eq!(cumulative_xp_for_level(0), 0);
}

// ─── level_from_xp ─────────────────────────────────────────────────

#[test]
fn level_from_xp_zero_is_1() {
    assert_eq!(level_from_xp(0), 1);
}

#[test]
fn level_from_xp_299_is_1() {
    assert_eq!(level_from_xp(299), 1);
}

#[test]
fn level_from_xp_300_is_2() {
    assert_eq!(level_from_xp(300), 2);
}

#[test]
fn level_from_xp_355000_is_20() {
    assert_eq!(level_from_xp(355_000), 20);
}

#[test]
fn level_from_xp_beyond_max_is_20() {
    assert_eq!(level_from_xp(u64::MAX), MAX_LEVEL);
}

#[test]
fn level_from_xp_mid_threshold() {
    // 2699 在 3 级范围（3 级 = 2700 XP 门槛以下）
    assert_eq!(level_from_xp(2699), 3);
    // DEFAULT_XP_THRESHOLDS[3] = 2700 是 4 级的门槛
    assert_eq!(level_from_xp(2700), 4);
    // 6499 已超过 2700 门槛，仍在 4 级范围（4 级 = 2700〜6499）
    assert_eq!(level_from_xp(6499), 4);
    // DEFAULT_XP_THRESHOLDS[4] = 6500 是 5 级的门槛
    assert_eq!(level_from_xp(6500), 5);
}

// ─── proficiency_bonus ─────────────────────────────────────────────

#[test]
fn proficiency_bonus_lv1_to_lv4_is_2() {
    for lv in 1..=4 {
        assert_eq!(proficiency_bonus(lv), 2, "lv{} 熟练加值应为 +2", lv);
    }
}

#[test]
fn proficiency_bonus_lv5_to_lv8_is_3() {
    for lv in 5..=8 {
        assert_eq!(proficiency_bonus(lv), 3, "lv{} 熟练加值应为 +3", lv);
    }
}

#[test]
fn proficiency_bonus_lv17_to_lv20_is_6() {
    for lv in 17..=20 {
        assert_eq!(proficiency_bonus(lv), 6, "lv{} 熟练加值应为 +6", lv);
    }
}

#[test]
fn proficiency_bonus_lv0_falls_back_to_2() {
    // 等级 0 的统一处理：返回 lv1 的基准值 2
    assert_eq!(proficiency_bonus(0), 2);
}

#[test]
fn proficiency_bonus_above_max_clamps() {
    assert_eq!(proficiency_bonus(MAX_LEVEL + 10), 6);
}

// ─── is_asi_level / next_asi_level / asi_levels ───────────────────

#[test]
fn is_asi_level_returns_true_for_standard_levels() {
    for &lv in &[4, 8, 12, 16, 19] {
        assert!(is_asi_level(lv), "lv{} 应为 ASI 等级", lv);
    }
}

#[test]
fn is_asi_level_returns_false_for_non_asi_levels() {
    for lv in [1, 2, 3, 5, 6, 7, 9, 10, 11, 13, 14, 15, 17, 18, 20] {
        assert!(!is_asi_level(lv), "lv{} 不应为 ASI 等级", lv);
    }
}

#[test]
fn next_asi_level_finds_next() {
    assert_eq!(next_asi_level(3), Some(4));
    assert_eq!(next_asi_level(7), Some(8));
    assert_eq!(next_asi_level(11), Some(12));
    assert_eq!(next_asi_level(15), Some(16));
    assert_eq!(next_asi_level(18), Some(19));
}

#[test]
fn next_asi_level_none_when_at_or_past_last() {
    assert_eq!(next_asi_level(19), None);
    assert_eq!(next_asi_level(20), None);
}

#[test]
fn asi_levels_returns_standard_five() {
    assert_eq!(asi_levels(), &[4, 8, 12, 16, 19]);
}
