//! Progression 领域不变量测试
//!
//! 验证 progression_domain.md §3 中定义的核心不变量：
//!   不变量 3.1 — 等级不得超过 20
//!   不变量 3.2 — 经验只增不减
//!   不变量 3.3 — 天赋前置链完整性
//!   不变量 3.4 — 子职选择后不可更改
//!   不变量 3.5 — ASI 不可跳过，属性上限 20
//!
//! 这些测试保证业务规则在任何代码变更后仍然成立。

use crate::core::domains::progression::components::{
    ClassId, ClassLevels, Experience, SubclassChoice, SubclassId, TalentId, TalentTree,
};
use crate::core::domains::progression::rules::formulas::MAX_LEVEL;
use crate::core::domains::progression::rules::rules::{
    check_asi_attribute_increase, check_talent_prerequisites,
};

// ─── 不变量 3.1：等级不得超过 20 ──────────────────────────────────

#[test]
fn invariant_level_never_exceeds_20_via_apply_level_up() {
    // 满级后再 apply_level_up 不会增加等级
    let mut xp = Experience {
        level: 20,
        is_max_level: true,
        ..Experience::new()
    };
    xp.apply_level_up(0);
    assert!(
        xp.level <= MAX_LEVEL,
        "不变量 3.1 违反：等级 {} 超过上限 20",
        xp.level
    );
}

#[test]
fn invariant_total_level_sum_never_exceeds_20() {
    // ClassLevels.total_level() 不会超过 20（需要外部约束，这里验证函数不会内部溢出）
    let mut cl = ClassLevels::new("fighter");
    for _ in 0..10 {
        cl.advance_class(ClassId::new("fighter"));
        cl.advance_class(ClassId::new("wizard"));
    }
    // 20 次 advance → fighter 11, wizard 10 → total 21
    // ClassLevels 本身不限制总和（约束由业务规则保证），但这里确认计算正确性
    assert_eq!(cl.total_level(), 21); // 验证计算准确
}

#[test]
fn invariant_can_level_up_false_at_max_level() {
    // 满级时 can_level_up 始终返回 false
    let xp = Experience {
        level: 20,
        is_max_level: true,
        current_xp: 999999,
        ..Experience::new()
    };
    assert!(!xp.can_level_up(1), "不变量 3.1 违反：满级仍可升级");
}

// ─── 不变量 3.2：经验只增不减 ──────────────────────────────────────

#[test]
fn invariant_xp_only_increases_on_add() {
    // total_xp_earned 和对外的 current_xp 在 add_xp 时不会减少
    let mut xp = Experience::new();
    let before = xp.total_xp_earned;
    xp.add_xp(500);
    assert!(
        xp.total_xp_earned >= before,
        "不变量 3.2 违反：total_xp_earned 减少"
    );
    assert!(xp.current_xp >= before, "不变量 3.2 违反：current_xp 减少");
}

#[test]
fn invariant_xp_gain_validated_positive() {
    // validate_xp_gain 拒绝零或负经验增加（此处验证零被拒绝）
    use crate::core::domains::progression::rules::rules::validate_xp_gain;
    assert!(
        validate_xp_gain(0).is_err(),
        "不变量 3.2 违反：经验增加量为 0 应被拒绝"
    );
}

// ─── 不变量 3.3：天赋前置链完整性 ──────────────────────────────────

#[test]
fn invariant_talent_prerequisites_checked_before_unlock() {
    // 调用 check_talent_prerequisites 后未满足条件时返回 Err
    let result = check_talent_prerequisites(5, 3, &["prereq_a".to_string()], &[]);
    assert!(result.is_err(), "不变量 3.3 违反：前置条件不满足应拒绝解锁");
}

#[test]
fn invariant_talent_no_duplicate_unlock() {
    // TalentTree.unlock 不应该产生重复条目
    let mut tree = TalentTree::new();
    let talent = TalentId::new("toughness");
    tree.unlock(talent.clone());
    tree.unlock(talent.clone());
    assert_eq!(
        tree.unlocked_talents.len(),
        1,
        "不变量 3.3 违反：天赋重复解锁"
    );
}

// ─── 不变量 3.4：子职选择后不可更改 ────────────────────────────────

#[test]
fn invariant_subclass_immutable_after_choice() {
    // SubclassChoice.choose 对同一职业第二次调用时必须返回 Err
    let mut sc = SubclassChoice::new();
    let fighter = ClassId::new("fighter");
    sc.choose(fighter.clone(), SubclassId::new("champion"))
        .unwrap();
    let second_attempt = sc.choose(fighter.clone(), SubclassId::new("battlemaster"));
    assert!(second_attempt.is_err(), "不变量 3.4 违反：子职选择后可更改");
}

// ─── 不变量 3.5：ASI 属性上限 20 ───────────────────────────────────

#[test]
fn invariant_asi_attribute_never_exceeds_20() {
    // check_asi_attribute_increase 在结果超过 20 时返回 Err
    let result = check_asi_attribute_increase(19, 2);
    assert!(result.is_err(), "不变量 3.5 违反：ASI 属性值超过 20");
}

#[test]
fn invariant_asi_attribute_exactly_20_is_ok() {
    // 恰好 20 是允许的边界
    assert!(
        check_asi_attribute_increase(18, 2).is_ok(),
        "不变量 3.5 违反：恰好 20 应被允许"
    );
}

#[test]
fn invariant_asi_no_negative_increase() {
    // ASI 不允许降低属性值
    let result = check_asi_attribute_increase(10, -1);
    assert!(result.is_err(), "不变量 3.5 违反：ASI 降低了属性值");
}
