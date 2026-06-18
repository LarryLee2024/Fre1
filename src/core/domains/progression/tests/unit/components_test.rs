//! 成长养成领域 Component — 行为测试
//!
//! 覆盖 components.rs 中所有 ECS Component 的方法在常规/边界/错误场景下的行为。
//! 规则依据：progression_domain.md §3 不变量

use crate::core::domains::progression::components::{
    ClassId, ClassLevels, Experience, LevelProgressionTable, SubclassChoice, SubclassId, TalentId,
    TalentTree,
};

// ─── Experience ────────────────────────────────────────────────────

#[test]
fn experience_new_creates_lv1() {
    let xp = Experience::new();
    assert_eq!(xp.level, 1);
    assert_eq!(xp.current_xp, 0);
    assert_eq!(xp.total_xp_earned, 0);
    assert!(!xp.is_max_level);
}

#[test]
fn experience_add_xp_increases_both_counters() {
    let mut xp = Experience::new();
    let total = xp.add_xp(500);
    assert_eq!(xp.current_xp, 500);
    assert_eq!(total, 500);
}

#[test]
fn experience_add_xp_at_max_level_noop() {
    let mut xp = Experience {
        level: 20,
        is_max_level: true,
        ..Experience::new()
    };
    let total = xp.add_xp(1000);
    assert_eq!(xp.current_xp, 0); // 满级不再积累
    assert_eq!(total, 0); // total_xp_earned 不变
}

#[test]
fn experience_add_xp_accumulates_total() {
    let mut xp = Experience::new();
    xp.add_xp(300);
    xp.add_xp(500);
    assert_eq!(xp.total_xp_earned, 800);
    assert_eq!(xp.current_xp, 800);
}

#[test]
fn experience_can_level_up_when_xp_sufficient() {
    let xp = Experience {
        current_xp: 300,
        ..Experience::new()
    };
    assert!(xp.can_level_up(300));
}

#[test]
fn experience_can_level_up_when_xp_insufficient() {
    let xp = Experience {
        current_xp: 200,
        ..Experience::new()
    };
    assert!(!xp.can_level_up(300));
}

#[test]
fn experience_can_level_up_at_max_level_always_false() {
    let xp = Experience {
        current_xp: 999999,
        level: 20,
        is_max_level: true,
        ..Experience::new()
    };
    assert!(!xp.can_level_up(0));
}

#[test]
fn experience_apply_level_up_deducts_xp_and_increments_level() {
    let mut xp = Experience {
        current_xp: 500,
        level: 1,
        ..Experience::new()
    };
    xp.apply_level_up(300);
    assert_eq!(xp.level, 2);
    assert_eq!(xp.current_xp, 200);
    assert!(!xp.is_max_level);
}

#[test]
fn experience_apply_level_up_reaches_max() {
    let mut xp = Experience {
        current_xp: 50000,
        level: 19,
        ..Experience::new()
    };
    xp.apply_level_up(50000);
    assert_eq!(xp.level, 20);
    assert!(xp.is_max_level);
}

#[test]
fn experience_apply_level_up_at_max_noop() {
    let mut xp = Experience {
        level: 20,
        is_max_level: true,
        ..Experience::new()
    };
    xp.apply_level_up(100);
    assert_eq!(xp.level, 20);
    assert_eq!(xp.current_xp, 0);
}

#[test]
fn experience_default_equals_new() {
    assert_eq!(Experience::default(), Experience::new());
}

// ─── ClassLevels ───────────────────────────────────────────────────

#[test]
fn class_levels_new_creates_single_class() {
    let cl = ClassLevels::new("fighter");
    assert_eq!(cl.total_level(), 1);
    assert_eq!(cl.level_in_class(&ClassId::new("fighter")), 1);
}

#[test]
fn class_levels_total_level_sums_all_classes() {
    let mut cl = ClassLevels::new("fighter");
    cl.advance_class(ClassId::new("wizard"));
    cl.advance_class(ClassId::new("wizard"));
    // fighter: 1, wizard: 2 → total 3
    assert_eq!(cl.total_level(), 3);
}

#[test]
fn class_levels_advance_class_increments_existing() {
    let mut cl = ClassLevels::new("fighter");
    cl.advance_class(ClassId::new("fighter"));
    assert_eq!(cl.level_in_class(&ClassId::new("fighter")), 2);
}

#[test]
fn class_levels_advance_class_adds_new() {
    let mut cl = ClassLevels::new("fighter");
    cl.advance_class(ClassId::new("cleric"));
    assert_eq!(cl.level_in_class(&ClassId::new("cleric")), 1);
    assert_eq!(cl.total_level(), 2);
}

#[test]
fn class_levels_level_in_class_unknown_returns_zero() {
    let cl = ClassLevels::new("fighter");
    assert_eq!(cl.level_in_class(&ClassId::new("nonexistent")), 0);
}

#[test]
fn class_levels_has_class_true_for_known() {
    let cl = ClassLevels::new("fighter");
    assert!(cl.has_class(&ClassId::new("fighter")));
    assert!(!cl.has_class(&ClassId::new("wizard")));
}

// ─── TalentTree ────────────────────────────────────────────────────

#[test]
fn talent_tree_new_empty() {
    let tree = TalentTree::new();
    assert!(tree.unlocked_talents.is_empty());
    assert_eq!(tree.available_points, 0);
}

#[test]
fn talent_tree_unlock_adds_talent() {
    let mut tree = TalentTree::new();
    tree.unlock(TalentId::new("toughness"));
    assert!(tree.is_unlocked(&TalentId::new("toughness")));
}

#[test]
fn talent_tree_unlock_duplicate_noop() {
    let mut tree = TalentTree::new();
    tree.unlock(TalentId::new("toughness"));
    tree.unlock(TalentId::new("toughness"));
    assert_eq!(tree.unlocked_talents.len(), 1);
}

#[test]
fn talent_tree_is_unlocked_false_for_unknown() {
    let tree = TalentTree::new();
    assert!(!tree.is_unlocked(&TalentId::new("unknown")));
}

#[test]
fn talent_tree_add_points_increases() {
    let mut tree = TalentTree::new();
    tree.add_points(5);
    assert_eq!(tree.available_points, 5);
}

#[test]
fn talent_tree_spend_point_success() {
    let mut tree = TalentTree::new();
    tree.add_points(2);
    assert!(tree.spend_point());
    assert_eq!(tree.available_points, 1);
}

#[test]
fn talent_tree_spend_point_fails_when_zero() {
    let mut tree = TalentTree::new();
    assert!(!tree.spend_point());
}

#[test]
fn talent_tree_default_equals_new() {
    assert_eq!(TalentTree::default(), TalentTree::new());
}

// ─── SubclassChoice ────────────────────────────────────────────────

#[test]
fn subclass_choice_new_empty() {
    let sc = SubclassChoice::new();
    assert!(sc.choices.is_empty());
}

#[test]
fn subclass_choice_choose_inserts() {
    let mut sc = SubclassChoice::new();
    assert!(
        sc.choose(ClassId::new("fighter"), SubclassId::new("champion"))
            .is_ok()
    );
    assert_eq!(
        sc.get(&ClassId::new("fighter")),
        Some(&SubclassId::new("champion"))
    );
}

#[test]
fn subclass_choice_choose_twice_same_class_fails() {
    let mut sc = SubclassChoice::new();
    sc.choose(ClassId::new("fighter"), SubclassId::new("champion"))
        .unwrap();
    let result = sc.choose(ClassId::new("fighter"), SubclassId::new("battlemaster"));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already has subclass"));
}

#[test]
fn subclass_choice_get_none_for_unselected() {
    let sc = SubclassChoice::new();
    assert_eq!(sc.get(&ClassId::new("fighter")), None);
}

#[test]
fn subclass_choice_different_classes_independent() {
    let mut sc = SubclassChoice::new();
    sc.choose(ClassId::new("fighter"), SubclassId::new("champion"))
        .unwrap();
    assert!(
        sc.choose(ClassId::new("wizard"), SubclassId::new("evocation"))
            .is_ok()
    );
}

// ─── LevelProgressionTable ─────────────────────────────────────────

#[test]
fn progression_table_default_has_20_levels() {
    let table = LevelProgressionTable::default();
    assert_eq!(table.max_level, 20);
    assert_eq!(table.exp_thresholds.len(), 20);
    assert_eq!(table.proficiency_by_level.len(), 20);
}

#[test]
fn progression_table_xp_for_level_1_to_2() {
    let table = LevelProgressionTable::default();
    assert_eq!(table.xp_for_level(1), 0); // 起始等级
    assert_eq!(table.xp_for_level(2), 300); // 到 2 级需 300 XP
}

#[test]
fn progression_table_xp_for_level_beyond_max() {
    let table = LevelProgressionTable::default();
    assert_eq!(table.xp_for_level(21), u64::MAX);
}

#[test]
fn progression_table_xp_range_for_level() {
    let table = LevelProgressionTable::default();
    // 1 级范围: (0, 300) = (xp_for_level(1), xp_for_level(2))
    assert_eq!(table.xp_range_for_level(1), (0, 300));
}

#[test]
fn progression_table_level_from_xp() {
    let table = LevelProgressionTable::default();
    assert_eq!(table.level_from_xp(0), 1);
    assert_eq!(table.level_from_xp(300), 2);
    assert_eq!(table.level_from_xp(355_000), 20);
}

#[test]
fn progression_table_proficiency_bonus() {
    let table = LevelProgressionTable::default();
    assert_eq!(table.proficiency_bonus(1), 2);
    assert_eq!(table.proficiency_bonus(17), 6);
}

#[test]
fn progression_table_is_asi_level() {
    let table = LevelProgressionTable::default();
    assert!(table.is_asi_level(4));
    assert!(!table.is_asi_level(5));
}
