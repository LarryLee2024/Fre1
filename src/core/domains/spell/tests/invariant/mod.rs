//! Spell Domain — 不变量测试
//!
//! 验证 docs/02-domain/domains/spell_domain.md §3 定义的不变量。

use crate::core::domains::spell::components::{
    CastingTime, SpellComponents, SpellDef, SpellDefId, SpellDuration, SpellLevel, SpellRange,
    SpellSlotEntry, SpellSlotPool,
};
use crate::core::domains::spell::rules::{
    calc_concentration_dc, check_components, check_concentration, check_slot_available,
    check_upcast, concentration_save, proficiency_bonus_for_level,
};

/// 不变量 3.1：法术位不可透支 — used <= total 始终成立（check_slot_available 保证）。
#[test]
fn slot_never_overdrawn() {
    let pool = pool_with_level(2, 2); // all used
    assert!(
        check_slot_available(&pool, SpellLevel::L1, &SpellDefId::new("spl_test")).is_err(),
        "no available slots"
    );
}

/// 不变量 3.1b：戏法不消耗法术位。
#[test]
fn cantrip_no_slot_required() {
    let empty_pool = pool_with_level(0, 0);
    assert!(
        check_slot_available(
            &empty_pool,
            SpellLevel::Cantrip,
            &SpellDefId::new("spl_cantrip")
        )
        .is_ok()
    );
}

/// 不变量 3.2：专注唯一性 — 不能同时维持两个专注法术。
#[test]
fn concentration_unique() {
    let current = crate::core::domains::spell::components::Concentration::new(
        SpellDefId::new("spl_000001"),
        10,
        2,
    );
    let result = check_concentration(Some(&current), &SpellDefId::new("spl_000002"));
    assert!(result.is_err(), "second concentration must be rejected");
}

/// 不变量 3.3：施法组件必检查 — 缺少组件施法失败。
#[test]
fn components_must_be_checked() {
    let comp = SpellComponents {
        verbal: true,
        somatic: false,
        material: None,
    };
    assert!(
        check_components(&comp, true, false, false).is_err(),
        "silenced + verbal component → fail"
    );
}

/// 不变量 3.4：升环检查 — can_upcast=false 时不可升环。
#[test]
fn upcast_not_allowed_rejected() {
    let spell = SpellDef {
        id: SpellDefId::new("spl_upcast"),
        name_key: "test".into(),
        desc_key: "test".into(),
        level: SpellLevel::L2,
        casting_time: CastingTime::Action,
        components: SpellComponents {
            verbal: true,
            somatic: true,
            material: None,
        },
        range: SpellRange::Self_,
        duration: SpellDuration::Instant,
        requires_concentration: false,
        saving_throw: None,
        can_upcast: false,
        effects: vec![],
    };
    assert!(
        check_upcast(&spell, SpellLevel::L3).is_err(),
        "upcast not allowed → reject"
    );
}

/// 不变量 3.5：专注打断 DC = max(10, damage/2)。
#[test]
fn concentration_dc_formula() {
    assert_eq!(calc_concentration_dc(4), 10, "low damage → min DC 10");
    assert_eq!(calc_concentration_dc(30), 15, "high damage → DC = damage/2");
    assert_eq!(calc_concentration_dc(0), 10, "zero damage → min DC 10");
}

/// 不变量 3.5：专注打断检定 — 高 roll 通过，低 roll 失败。
#[test]
fn concentration_save_consistent() {
    let conc = crate::core::domains::spell::components::Concentration::new(
        SpellDefId::new("spl_conc"),
        10,
        2,
    );
    let (saved, dc) = concentration_save(&conc, 10, 15);
    assert!(saved);
    assert_eq!(dc, 10);

    let (failed, _) = concentration_save(&conc, 30, 3);
    assert!(!failed);
}

/// 衍生不变量：熟练加值随等级单调递增。
#[test]
fn proficiency_bonus_monotonic() {
    assert!(proficiency_bonus_for_level(1) <= proficiency_bonus_for_level(5));
    assert!(proficiency_bonus_for_level(5) <= proficiency_bonus_for_level(9));
    assert!(proficiency_bonus_for_level(9) <= proficiency_bonus_for_level(13));
    assert!(proficiency_bonus_for_level(13) <= proficiency_bonus_for_level(17));
}

// ─── Helpers ──────────────────────────────────────────────

fn pool_with_level(total: u32, used: u32) -> SpellSlotPool {
    let mut slots = Vec::new();
    for _ in 0..9 {
        slots.push(SpellSlotEntry { total, used });
    }
    SpellSlotPool {
        slots_by_level: slots,
    }
}
