//! Spell Domain — 单元测试
//!
//! 验证规则纯函数（rules.rs + formulas.rs）。
//! 不启动 App，不加载 Plugin，不依赖资源。

// ============================================================================
// rules.rs — check_spell_known
// ============================================================================

mod check_spell_known_tests {
    use crate::core::domains::spell::components::SpellDefId;
    use crate::core::domains::spell::rules::check_spell_known;

    #[test]
    fn known_spell_returns_ok() {
        let known = vec![SpellDefId::new("spl_000001"), SpellDefId::new("spl_000002")];
        let result = check_spell_known(&known, &SpellDefId::new("spl_000001"));
        assert!(result.is_ok());
    }

    #[test]
    fn unknown_spell_returns_error() {
        let known = vec![SpellDefId::new("spl_000001")];
        let result = check_spell_known(&known, &SpellDefId::new("spl_999999"));
        assert!(result.is_err());
    }
}

// ============================================================================
// rules.rs — check_spell_prepared
// ============================================================================

mod check_spell_prepared_tests {
    use crate::core::domains::spell::components::SpellDefId;
    use crate::core::domains::spell::rules::check_spell_prepared;

    #[test]
    fn prepared_spell_returns_ok() {
        let prepared = vec![SpellDefId::new("spl_000001")];
        let result = check_spell_prepared(&prepared, &SpellDefId::new("spl_000001"));
        assert!(result.is_ok());
    }

    #[test]
    fn unprepared_spell_returns_error() {
        let prepared = vec![];
        let result = check_spell_prepared(&prepared, &SpellDefId::new("spl_000001"));
        assert!(result.is_err());
    }
}

// ============================================================================
// rules.rs — check_components
// ============================================================================

mod check_components_tests {
    use crate::core::domains::spell::components::SpellComponents;
    use crate::core::domains::spell::failure::SpellFailure;
    use crate::core::domains::spell::rules::check_components;

    #[test]
    fn all_components_satisfied() {
        let components = SpellComponents {
            verbal: true,
            somatic: true,
            material: None,
        };
        assert!(check_components(&components, true, true, false).is_ok());
    }

    #[test]
    fn silenced_blocks_verbal() {
        let components = SpellComponents {
            verbal: true,
            somatic: false,
            material: None,
        };
        let result = check_components(&components, false, true, false);
        assert_eq!(result.unwrap_err(), SpellFailure::Silenced);
    }

    #[test]
    fn restrained_blocks_somatic() {
        let components = SpellComponents {
            verbal: false,
            somatic: true,
            material: None,
        };
        let result = check_components(&components, true, false, false);
        assert_eq!(result.unwrap_err(), SpellFailure::Restrained);
    }
}

// ============================================================================
// rules.rs — check_slot_available
// ============================================================================

mod check_slot_available_tests {
    use crate::core::domains::spell::components::{
        SpellDefId, SpellLevel, SpellSlotEntry, SpellSlotPool,
    };
    use crate::core::domains::spell::rules::check_slot_available;

    fn pool_with_level(total: u32, used: u32) -> SpellSlotPool {
        let mut slots = Vec::new();
        for _ in 0..9 {
            slots.push(SpellSlotEntry { total, used });
        }
        SpellSlotPool {
            slots_by_level: slots,
        }
    }

    #[test]
    fn sufficient_slots_returns_ok() {
        let pool = pool_with_level(2, 0);
        assert!(check_slot_available(&pool, SpellLevel::L1, &SpellDefId::new("spl_test")).is_ok());
    }

    #[test]
    fn insufficient_slots_returns_error() {
        let pool = pool_with_level(1, 1);
        let result = check_slot_available(&pool, SpellLevel::L1, &SpellDefId::new("spl_test"));
        assert!(result.is_err());
    }

    #[test]
    fn cantrip_never_requires_slots() {
        let pool = pool_with_level(0, 0);
        assert!(
            check_slot_available(&pool, SpellLevel::Cantrip, &SpellDefId::new("spl_cantrip"))
                .is_ok()
        );
    }
}

// ============================================================================
// rules.rs — check_concentration
// ============================================================================

mod check_concentration_tests {
    use crate::core::domains::spell::components::{Concentration, SpellDefId};
    use crate::core::domains::spell::rules::check_concentration;

    fn make_concentration() -> Concentration {
        Concentration::new(SpellDefId::new("spl_000001"), 10, 2)
    }

    #[test]
    fn no_conflict_returns_ok() {
        assert!(check_concentration(None, &SpellDefId::new("spl_000002")).is_ok());
    }

    #[test]
    fn conflicting_concentration_returns_error() {
        let current = make_concentration();
        let result = check_concentration(Some(&current), &SpellDefId::new("spl_000002"));
        assert!(result.is_err());
    }
}

// ============================================================================
// rules.rs — check_upcast
// ============================================================================

mod check_upcast_tests {
    use crate::core::domains::spell::components::{
        CastingTime, SpellComponents, SpellDef, SpellDefId, SpellDuration, SpellLevel, SpellRange,
    };
    use crate::core::domains::spell::rules::check_upcast;

    fn make_upcastable_spell() -> SpellDef {
        SpellDef {
            id: SpellDefId::new("spl_000001"),
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
            can_upcast: true,
            effects: vec![],
        }
    }

    #[test]
    fn valid_upcast_returns_ok() {
        let spell = make_upcastable_spell();
        assert!(check_upcast(&spell, SpellLevel::L3).is_ok());
    }

    #[test]
    fn upcast_not_allowed_returns_error() {
        let mut spell = make_upcastable_spell();
        spell.can_upcast = false;
        let result = check_upcast(&spell, SpellLevel::L3);
        assert!(result.is_err());
    }

    #[test]
    fn same_level_upcast_returns_error() {
        let spell = make_upcastable_spell();
        let result = check_upcast(&spell, SpellLevel::L2);
        assert!(result.is_err());
    }
}

// ============================================================================
// rules.rs — concentration_save
// ============================================================================

mod concentration_save_tests {
    use crate::core::domains::spell::components::{Concentration, SpellDefId};
    use crate::core::domains::spell::rules::concentration_save;

    fn make_concentration() -> Concentration {
        Concentration::new(SpellDefId::new("spl_000001"), 10, 2)
    }

    #[test]
    fn save_succeeds_with_high_roll() {
        let conc = make_concentration();
        let (saved, dc) = concentration_save(&conc, 10, 15, 10);
        assert!(saved);
        assert_eq!(dc, 10);
    }

    #[test]
    fn save_fails_with_low_roll() {
        let conc = make_concentration();
        let (saved, dc) = concentration_save(&conc, 30, 5, 10);
        assert!(!saved);
        assert_eq!(dc, 15);
    }

    #[test]
    fn dc_has_minimum_of_10() {
        let conc = make_concentration();
        let (_saved, dc) = concentration_save(&conc, 2, 10, 10);
        assert_eq!(dc, 10);
    }
}

// ============================================================================
// rules.rs — resolve_save
// ============================================================================

mod resolve_save_tests {
    use crate::core::domains::spell::components::SaveResult;
    use crate::core::domains::spell::rules::resolve_save;

    #[test]
    fn natural_20_always_succeeds() {
        assert_eq!(resolve_save(20, 0, 25), SaveResult::Success);
    }

    #[test]
    fn natural_1_always_fails() {
        assert_eq!(resolve_save(1, 10, 5), SaveResult::Failure);
    }

    #[test]
    fn roll_meets_dc_succeeds() {
        assert_eq!(resolve_save(15, 5, 20), SaveResult::Success);
    }

    #[test]
    fn roll_below_dc_fails() {
        assert_eq!(resolve_save(10, 2, 15), SaveResult::Failure);
    }
}

// ============================================================================
// formulas.rs — calc_save_dc
// ============================================================================

mod calc_save_dc_tests {
    use crate::core::domains::spell::rules::calc_save_dc;

    #[test]
    fn standard_calculation() {
        assert_eq!(calc_save_dc(2, 3, 0), 13);
    }

    #[test]
    fn with_other_bonuses() {
        assert_eq!(calc_save_dc(4, 5, 1), 18);
    }

    #[test]
    fn minimum_value() {
        assert_eq!(calc_save_dc(-5, -5, 0), 1);
    }
}

// ============================================================================
// formulas.rs — calc_concentration_dc
// ============================================================================

mod calc_concentration_dc_tests {
    use crate::core::domains::spell::rules::calc_concentration_dc;

    #[test]
    fn low_damage_uses_minimum() {
        assert_eq!(calc_concentration_dc(4, 10), 10);
    }

    #[test]
    fn high_damage_increases_dc() {
        assert_eq!(calc_concentration_dc(30, 10), 15);
    }

    #[test]
    fn zero_damage_uses_minimum() {
        assert_eq!(calc_concentration_dc(0, 10), 10);
    }
}

// ============================================================================
// formulas.rs — calc_upcast_bonus
// ============================================================================

mod calc_upcast_bonus_tests {
    use crate::core::domains::spell::rules::calc_upcast_bonus;

    #[test]
    fn multiple_levels_upcast() {
        assert_eq!(calc_upcast_bonus(2, 5, 2), 6);
    }

    #[test]
    fn no_upcast_returns_zero() {
        assert_eq!(calc_upcast_bonus(3, 3, 5), 0);
    }

    #[test]
    fn single_level_upcast() {
        assert_eq!(calc_upcast_bonus(1, 2, 3), 3);
    }
}

// ============================================================================
// formulas.rs — proficiency_bonus_for_level
// ============================================================================

mod proficiency_bonus_tests {
    use crate::core::domains::spell::rules::proficiency_bonus_for_level;

    #[test]
    fn level_1_to_4_is_plus_2() {
        assert_eq!(proficiency_bonus_for_level(1), 2);
        assert_eq!(proficiency_bonus_for_level(4), 2);
    }

    #[test]
    fn level_5_to_8_is_plus_3() {
        assert_eq!(proficiency_bonus_for_level(5), 3);
    }

    #[test]
    fn level_17_to_20_is_plus_6() {
        assert_eq!(proficiency_bonus_for_level(17), 6);
        assert_eq!(proficiency_bonus_for_level(20), 6);
    }

    #[test]
    fn out_of_range_defaults_to_2() {
        assert_eq!(proficiency_bonus_for_level(0), 2);
        assert_eq!(proficiency_bonus_for_level(21), 2);
    }
}
