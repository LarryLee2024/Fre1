use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectCategory, EffectDuration, EffectError,
    EffectInstance, EffectPeriod, EffectStage, RemovalReason, TickState,
};
use crate::core::capabilities::effect::mechanism::lifecycle::{
    ApplyResult, apply_effect, expire_effects, remove_effect_by_id, remove_effects_by_def,
    remove_effects_by_source, tick_durations,
};

// -- Helpers ---------------------------------------------------------------

fn make_instant_effect(id: &str) -> EffectInstance {
    EffectInstance::new(
        id,
        "eff_damage",
        "Damage",
        "caster_001",
        "target_001",
        EffectDuration::Instant,
        1,
    )
}

fn make_duration_effect(id: &str, turns: u32) -> EffectInstance {
    EffectInstance::new(
        id,
        "eff_poison",
        "Debuff",
        "caster_001",
        "target_001",
        EffectDuration::HasDuration {
            turns,
            calculation: DurationCalculation::Fixed,
        },
        1,
    )
}

fn make_infinite_effect(id: &str) -> EffectInstance {
    EffectInstance::new(
        id,
        "eff_aura",
        "Buff",
        "caster_001",
        "target_001",
        EffectDuration::Infinite,
        1,
    )
}

fn make_container() -> ActiveEffectContainer {
    ActiveEffectContainer::new()
}

// -- EffectInstance creation -----------------------------------------------

#[test]
fn instant_effect_starts_applying() {
    let effect = make_instant_effect("inst_001");
    assert_eq!(effect.stage, EffectStage::Applying);
    assert_eq!(effect.remaining_turns, 0);
}

#[test]
fn duration_effect_starts_applying() {
    let effect = make_duration_effect("dur_001", 3);
    assert_eq!(effect.stage, EffectStage::Applying);
    assert_eq!(effect.remaining_turns, 3);
}

#[test]
fn infinite_effect_starts_applying() {
    let effect = make_infinite_effect("inf_001");
    assert_eq!(effect.stage, EffectStage::Applying);
    assert_eq!(effect.remaining_turns, i64::MAX);
}

// -- Apply effect ----------------------------------------------------------

#[test]
fn apply_instant_effect_success() {
    let mut container = make_container();
    let effect = make_instant_effect("inst_001");
    let result = apply_effect(&mut container, effect);
    assert!(result.success);
    assert_eq!(container.count(), 0);
}

#[test]
fn apply_duration_effect_success() {
    let mut container = make_container();
    let effect = make_duration_effect("dur_001", 3);
    let result = apply_effect(&mut container, effect);
    assert!(result.success);
    assert_eq!(container.count(), 1);
}

#[test]
fn apply_infinite_effect_success() {
    let mut container = make_container();
    let effect = make_infinite_effect("inf_001");
    let result = apply_effect(&mut container, effect);
    assert!(result.success);
    assert_eq!(container.count(), 1);
}

#[test]
fn apply_duplicate_effect_rejected() {
    let mut container = make_container();
    let effect = make_duration_effect("dup_001", 3);
    let first = apply_effect(&mut container, effect);
    assert!(first.success);
    let second = make_duration_effect("dup_002", 3);
    let result = apply_effect(&mut container, second);
    assert!(!result.success);
    assert!(matches!(
        result.error,
        Some(EffectError::DuplicateEffect { .. })
    ));
}

#[test]
fn apply_effect_missing_source_rejected() {
    let mut container = make_container();
    let effect = EffectInstance::new(
        "no_source",
        "eff_test",
        "Test",
        "",
        "target_001",
        EffectDuration::Instant,
        1,
    );
    let result = apply_effect(&mut container, effect);
    assert!(!result.success);
    assert!(matches!(result.error, Some(EffectError::MissingSource(_))));
}

#[test]
fn apply_effect_slot_limit() {
    let mut container = ActiveEffectContainer::new().with_max_effects(1);
    let first = make_duration_effect("first", 3);
    let _ = apply_effect(&mut container, first);
    let second = EffectInstance::new(
        "second",
        "eff_other",
        "Buff",
        "caster_002",
        "target_001",
        EffectDuration::Infinite,
        1,
    );
    let result = apply_effect(&mut container, second);
    assert!(!result.success);
    assert!(matches!(
        result.error,
        Some(EffectError::SlotLimitReached { .. })
    ));
}

#[test]
fn apply_multiple_different_effects_ok() {
    let mut container = make_container();
    let a = make_duration_effect("a", 3);
    assert!(apply_effect(&mut container, a).success);
    let b = EffectInstance::new(
        "b",
        "eff_other",
        "Buff",
        "caster_002",
        "target_001",
        EffectDuration::Infinite,
        1,
    );
    assert!(apply_effect(&mut container, b).success);
    assert_eq!(container.count(), 2);
}

// -- Stage transitions -----------------------------------------------------

#[test]
fn transition_applying_to_active() {
    let mut effect = make_duration_effect("test", 3);
    assert!(effect.transition_to(EffectStage::Active).is_ok());
    assert_eq!(effect.stage, EffectStage::Active);
}

#[test]
fn transition_applying_to_removed() {
    let mut effect = make_instant_effect("test");
    assert!(effect.transition_to(EffectStage::Removed).is_ok());
    assert_eq!(effect.stage, EffectStage::Removed);
}

#[test]
fn transition_active_to_expiring() {
    let mut effect = make_duration_effect("test", 3);
    let _ = effect.transition_to(EffectStage::Active);
    assert!(effect.transition_to(EffectStage::Expiring).is_ok());
}

#[test]
fn transition_expiring_to_removed() {
    let mut effect = make_duration_effect("test", 3);
    let _ = effect.transition_to(EffectStage::Active);
    let _ = effect.transition_to(EffectStage::Expiring);
    assert!(effect.transition_to(EffectStage::Removed).is_ok());
}

#[test]
fn invalid_transition_active_to_applying() {
    let mut effect = make_duration_effect("test", 3);
    let _ = effect.transition_to(EffectStage::Active);
    let result = effect.transition_to(EffectStage::Applying);
    assert!(result.is_err());
}

#[test]
fn invalid_transition_removed_to_any() {
    let mut effect = make_instant_effect("test");
    let _ = effect.transition_to(EffectStage::Removed);
    assert!(effect.transition_to(EffectStage::Active).is_err());
}

// -- Duration ticking ------------------------------------------------------

#[test]
fn tick_duration_reduces_remaining_turns() {
    let mut container = make_container();
    let effect = make_duration_effect("test", 3);
    let _ = apply_effect(&mut container, effect);
    let result = tick_durations(&mut container, 1, 2);
    let instance = container.find_by_id("test").unwrap();
    assert_eq!(instance.remaining_turns, 2);
    assert!(result.ticked.is_empty());
    assert!(result.expired.is_empty());
}

#[test]
fn tick_duration_to_zero_triggers_expiring() {
    let mut container = make_container();
    let effect = make_duration_effect("test", 2);
    let _ = apply_effect(&mut container, effect);
    tick_durations(&mut container, 2, 2);
    let instance = container.find_by_id("test").unwrap();
    assert_eq!(instance.stage, EffectStage::Expiring);
    assert_eq!(instance.remaining_turns, 0);
}

#[test]
fn tick_duration_beyond_zero_clamps() {
    let mut container = make_container();
    let effect = make_duration_effect("test", 1);
    let _ = apply_effect(&mut container, effect);
    tick_durations(&mut container, 5, 2);
    let instance = container.find_by_id("test").unwrap();
    assert_eq!(instance.remaining_turns, 0);
    assert_eq!(instance.stage, EffectStage::Expiring);
}

#[test]
fn tick_infinite_does_not_expire() {
    let mut container = make_container();
    let effect = make_infinite_effect("test");
    let _ = apply_effect(&mut container, effect);
    tick_durations(&mut container, 100, 2);
    let instance = container.find_by_id("test").unwrap();
    assert_eq!(instance.stage, EffectStage::Active);
}

#[test]
fn tick_paused_effect_skipped() {
    let mut container = make_container();
    let mut effect = make_duration_effect("test", 3);
    effect.paused = true;
    let _ = apply_effect(&mut container, effect);
    tick_durations(&mut container, 2, 2);
    let instance = container.find_by_id("test").unwrap();
    assert_eq!(instance.remaining_turns, 3);
}

// -- Periodic tick ---------------------------------------------------------

#[test]
fn periodic_tick_triggers() {
    let mut container = make_container();
    let period = EffectPeriod::new(1).unwrap();
    let effect = make_duration_effect("dot", 5).with_period(period);
    let _ = apply_effect(&mut container, effect);
    let result = tick_durations(&mut container, 1, 2);
    assert!(result.ticked.contains(&"dot".to_string()));
}

#[test]
fn periodic_tick_not_at_wrong_interval() {
    let mut container = make_container();
    let period = EffectPeriod::new(2).unwrap();
    let effect = make_duration_effect("dot", 5).with_period(period);
    let _ = apply_effect(&mut container, effect);
    let result = tick_durations(&mut container, 1, 2);
    assert!(result.ticked.is_empty());
}

#[test]
fn periodic_tick_max_ticks() {
    let mut container = make_container();
    let period = EffectPeriod::new(1).unwrap().with_max_ticks(2).unwrap();
    let effect = make_duration_effect("dot", 10).with_period(period);
    let _ = apply_effect(&mut container, effect);
    let r1 = tick_durations(&mut container, 1, 2);
    assert_eq!(r1.ticked.len(), 1);
    let r2 = tick_durations(&mut container, 1, 3);
    assert_eq!(r2.ticked.len(), 1);
    let r3 = tick_durations(&mut container, 1, 4);
    assert!(r3.ticked.is_empty());
}

// -- Expire effects --------------------------------------------------------

#[test]
fn expire_effects_moves_to_removed() {
    let mut container = make_container();
    let effect = make_duration_effect("test", 1);
    let _ = apply_effect(&mut container, effect);
    tick_durations(&mut container, 1, 2);
    let expired = expire_effects(&mut container);
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0], "test");
    let instance = container.find_by_id("test").unwrap();
    assert_eq!(instance.stage, EffectStage::Removed);
}

#[test]
fn expire_only_expiring_effects() {
    let mut container = make_container();
    let _ = apply_effect(&mut container, make_duration_effect("a", 3));
    let mut b = make_duration_effect("b", 1);
    b.source_entity = "caster_002".into();
    let _ = apply_effect(&mut container, b);
    tick_durations(&mut container, 1, 2);
    let expired = expire_effects(&mut container);
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0], "b");
}

// -- Remove effects --------------------------------------------------------

#[test]
fn remove_by_id_success() {
    let mut container = make_container();
    let effect = make_duration_effect("test", 3);
    let _ = apply_effect(&mut container, effect);
    let removed = remove_effect_by_id(&mut container, "test", &RemovalReason::Dispelled);
    assert!(removed.is_ok());
    assert_eq!(container.count(), 0);
}

#[test]
fn remove_by_id_not_found() {
    let mut container = make_container();
    let result = remove_effect_by_id(&mut container, "nonexistent", &RemovalReason::Manual);
    assert!(result.is_err());
}

#[test]
fn remove_undispellable_rejected() {
    let mut container = make_container();
    let mut effect = make_duration_effect("test", 3);
    effect.dispellable = false;
    let _ = apply_effect(&mut container, effect);
    let result = remove_effect_by_id(&mut container, "test", &RemovalReason::Dispelled);
    assert!(result.is_err());
}

#[test]
fn remove_undispellable_allowed_when_forced() {
    let mut container = make_container();
    let mut effect = make_duration_effect("test", 3);
    effect.dispellable = false;
    let _ = apply_effect(&mut container, effect);
    let result = remove_effect_by_id(&mut container, "test", &RemovalReason::Forced);
    assert!(result.is_ok());
}

#[test]
fn remove_by_source() {
    let mut container = make_container();
    let a = make_duration_effect("a", 3);
    let _ = apply_effect(&mut container, a);
    let b = EffectInstance::new(
        "b",
        "eff_other",
        "Buff",
        "caster_002",
        "target_001",
        EffectDuration::Infinite,
        1,
    );
    let _ = apply_effect(&mut container, b);
    let removed =
        remove_effects_by_source(&mut container, "caster_001", &RemovalReason::SourceDied);
    assert_eq!(removed.len(), 1);
    assert_eq!(container.count(), 1);
}

#[test]
fn remove_by_def() {
    let mut container = make_container();
    let a = make_duration_effect("a", 3);
    let _ = apply_effect(&mut container, a);
    let mut b = make_duration_effect("b", 3);
    b.source_entity = "caster_002".into();
    let _ = apply_effect(&mut container, b);
    let removed = remove_effects_by_def(&mut container, "eff_poison", &RemovalReason::Manual);
    assert_eq!(removed.len(), 2);
    assert_eq!(container.count(), 0);
}

// -- Container queries -----------------------------------------------------

#[test]
fn container_find_by_def() {
    let mut container = make_container();
    let a = make_duration_effect("a", 3);
    let _ = apply_effect(&mut container, a);
    let found = container.find_by_def("eff_poison");
    assert_eq!(found.len(), 1);
}

#[test]
fn container_find_by_source() {
    let mut container = make_container();
    let _ = apply_effect(&mut container, make_duration_effect("a", 3));
    let found = container.find_by_source("caster_001");
    assert_eq!(found.len(), 1);
}

#[test]
fn container_is_empty() {
    let container = make_container();
    assert!(container.is_empty());
}

#[test]
fn container_get_tickable() {
    let mut container = make_container();
    let period = EffectPeriod::new(1).unwrap();
    let effect = make_duration_effect("dot", 5).with_period(period);
    let _ = apply_effect(&mut container, effect);
    let tickable = container.get_tickable();
    assert_eq!(tickable.len(), 1);
}

#[test]
fn container_has_duplicate() {
    let mut container = make_container();
    let _ = apply_effect(&mut container, make_duration_effect("a", 3));
    assert!(container.has_duplicate("eff_poison", "caster_001"));
    assert!(!container.has_duplicate("eff_other", "caster_001"));
}

// -- TickState -------------------------------------------------------------

#[test]
fn tick_state_advance_triggers() {
    let period = EffectPeriod::new(2).unwrap();
    let mut state = TickState::new(&period);
    assert!(!state.advance(1));
    assert!(state.advance(1));
    assert_eq!(state.tick_count, 1);
    assert_eq!(state.remaining_turns, 2);
}

#[test]
fn tick_state_max_ticks_stops() {
    let period = EffectPeriod::new(1).unwrap().with_max_ticks(2).unwrap();
    let mut state = TickState::new(&period);
    assert!(state.advance(1));
    assert!(state.advance(1));
    assert!(!state.advance(1));
    assert_eq!(state.tick_count, 2);
}

#[test]
fn tick_state_has_more() {
    let period = EffectPeriod::new(1).unwrap().with_max_ticks(3).unwrap();
    let mut state = TickState::new(&period);
    assert!(state.has_more());
    state.tick_count = 3;
    assert!(!state.has_more());
}

// -- EffectStage helpers ---------------------------------------------------

#[test]
fn stage_is_active() {
    assert!(EffectStage::Applying.is_active());
    assert!(EffectStage::Active.is_active());
    assert!(!EffectStage::Expiring.is_active());
    assert!(!EffectStage::Removed.is_active());
}

#[test]
fn stage_can_tick() {
    assert!(!EffectStage::Applying.can_tick());
    assert!(EffectStage::Active.can_tick());
    assert!(!EffectStage::Expiring.can_tick());
    assert!(!EffectStage::Removed.can_tick());
}

#[test]
fn stage_name() {
    assert_eq!(EffectStage::Active.name(), "Active");
    assert_eq!(EffectStage::Removed.name(), "Removed");
}

// -- EffectDuration helpers ------------------------------------------------

#[test]
fn duration_is_instant() {
    assert!(EffectDuration::Instant.is_instant());
    assert!(
        !EffectDuration::HasDuration {
            turns: 3,
            calculation: DurationCalculation::Fixed
        }
        .is_instant()
    );
}

#[test]
fn duration_initial_remaining() {
    assert_eq!(EffectDuration::Instant.initial_remaining_turns(), 0);
    assert_eq!(
        EffectDuration::HasDuration {
            turns: 5,
            calculation: DurationCalculation::Fixed
        }
        .initial_remaining_turns(),
        5
    );
    assert_eq!(EffectDuration::Infinite.initial_remaining_turns(), i64::MAX);
}

// -- ApplyResult helpers ---------------------------------------------------

#[test]
fn apply_result_success() {
    let r = ApplyResult::success("inst_001");
    assert!(r.success);
    assert_eq!(r.instance_id, Some("inst_001".into()));
    assert!(r.error.is_none());
}

#[test]
fn apply_result_failure() {
    let r = ApplyResult::failure(EffectError::SlotLimitReached { current: 5, max: 5 });
    assert!(!r.success);
    assert!(r.instance_id.is_none());
    assert!(r.error.is_some());
}

// -- EffectPeriod validation -----------------------------------------------

#[test]
fn period_invalid_interval() {
    let result = EffectPeriod::new(0);
    assert!(result.is_err());
}

#[test]
fn period_invalid_max_ticks() {
    let period = EffectPeriod::new(1).unwrap();
    let result = period.with_max_ticks(0);
    assert!(result.is_err());
}

#[test]
fn period_valid() {
    let period = EffectPeriod::new(2).unwrap().with_max_ticks(5).unwrap();
    assert_eq!(period.interval_turns, 2);
    assert_eq!(period.max_ticks, Some(5));
}

// -- RemovalReason ---------------------------------------------------------

#[test]
fn removal_reason_name() {
    assert_eq!(RemovalReason::Expired.name(), "Expired");
    assert_eq!(RemovalReason::Dispelled.name(), "Dispelled");
    assert_eq!(RemovalReason::SourceDied.name(), "SourceDied");
}

// -- EffectCategory --------------------------------------------------------

#[test]
fn category_name() {
    assert_eq!(EffectCategory::Buff.name(), "Buff");
    assert_eq!(EffectCategory::Debuff.name(), "Debuff");
    assert_eq!(EffectCategory::Custom("test".into()).name(), "test");
}

// -- EffectInstance builder ------------------------------------------------

#[test]
fn instance_with_period() {
    let period = EffectPeriod::new(1).unwrap();
    let effect = make_duration_effect("test", 3).with_period(period);
    assert!(effect.tick_state.is_some());
}

#[test]
fn instance_with_modifiers() {
    let effect = make_duration_effect("test", 3).with_modifiers(2);
    assert_eq!(effect.modifier_count, 2);
}

#[test]
fn instance_undispellable() {
    let effect = make_duration_effect("test", 3).with_undispellable();
    assert!(!effect.dispellable);
}

#[test]
fn instance_with_stack() {
    let effect = make_duration_effect("test", 3).with_stack(3);
    assert_eq!(effect.stack_count, 3);
}

// -- EffectError Display ---------------------------------------------------

#[test]
fn error_display() {
    let err = EffectError::DuplicateEffect {
        def_id: "eff_test".into(),
        detail: "already active".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("eff_test"));
}
