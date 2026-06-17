use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectDuration, EffectInstance, EffectPeriod,
    EffectStage,
};

use super::facade::*;

fn make_test_container() -> ActiveEffectContainer {
    ActiveEffectContainer::new()
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

fn make_periodic_effect(id: &str, turns: u32, interval: u32) -> EffectInstance {
    let period = EffectPeriod::new(interval).unwrap();
    EffectInstance::new(
        id,
        "eff_dot",
        "Damage",
        "caster_001",
        "target_001",
        EffectDuration::HasDuration {
            turns,
            calculation: DurationCalculation::Fixed,
        },
        1,
    )
    .with_period(period)
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

fn push_active(container: &mut ActiveEffectContainer, effect: EffectInstance) {
    let mut e = effect;
    e.stage = EffectStage::Active;
    let _ = container.effects.push(e);
}

#[test]
fn empty_container_has_no_active_effects() {
    let container = make_test_container();
    assert!(!has_active_effects(&container));
    assert_eq!(active_effect_count(&container), 0);
}

#[test]
fn tick_and_expire_on_empty_container_succeeds() {
    let mut container = make_test_container();
    let outcome = tick_and_expire(&mut container, 1);
    assert!(outcome.ticked.is_empty());
    assert!(outcome.expired.is_empty());
    assert_eq!(outcome.error_count, 0);
}

#[test]
fn has_active_effects_with_duration_effect() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_001", 3));
    assert!(has_active_effects(&container));
    assert_eq!(active_effect_count(&container), 1);
}

#[test]
fn has_active_effect_by_def_finds_matching() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_001", 3));
    assert!(has_active_effect_by_def(&container, "eff_poison"));
    assert!(!has_active_effect_by_def(&container, "eff_nonexistent"));
}

#[test]
fn tick_decrements_remaining_turns() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_001", 3));

    let _ = tick_all_effects(&mut container, 1);

    assert_eq!(
        container.effects[0].remaining_turns, 2,
        "remaining_turns should decrease by 1"
    );
}

#[test]
fn tick_marks_expired_when_duration_depleted() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_002", 1));

    let outcome = tick_all_effects(&mut container, 1);

    assert_eq!(container.effects[0].remaining_turns, 0);
    assert_eq!(container.effects[0].stage, EffectStage::Expiring);
    assert_eq!(outcome.expired.len(), 1);
    assert!(outcome.expired.contains(&"dur_002".to_string()));
}

#[test]
fn tick_triggers_periodic_at_interval() {
    let mut container = make_test_container();
    push_active(&mut container, make_periodic_effect("dot_001", 5, 2));

    let outcome1 = tick_all_effects(&mut container, 1);
    assert!(outcome1.ticked.is_empty(), "interval=2, not yet reached");

    let outcome2 = tick_all_effects(&mut container, 2);
    assert_eq!(outcome2.ticked.len(), 1);
    assert!(outcome2.ticked.contains(&"dot_001".to_string()));
}

#[test]
fn tick_no_tick_before_interval() {
    let mut container = make_test_container();
    push_active(&mut container, make_periodic_effect("dot_002", 5, 3));

    let outcome = tick_all_effects(&mut container, 1);

    assert!(outcome.ticked.is_empty());
}

#[test]
fn infinite_effect_never_expires() {
    let mut container = make_test_container();
    push_active(&mut container, make_infinite_effect("inf_001"));

    let outcome = tick_all_effects(&mut container, 1000);

    assert!(outcome.expired.is_empty());
    assert_eq!(container.effects[0].remaining_turns, i64::MAX);
    assert_eq!(container.effects[0].stage, EffectStage::Active);
}

#[test]
fn paused_effect_does_not_tick() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("paused_001", 3));
    container.effects[0].paused = true;

    let outcome = tick_all_effects(&mut container, 1);

    assert_eq!(container.effects[0].remaining_turns, 3);
    assert!(outcome.ticked.is_empty());
    assert!(outcome.expired.is_empty());
}

#[test]
fn multiple_effects_tick_independently() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("a", 2));
    push_active(&mut container, make_duration_effect("b", 5));
    push_active(&mut container, make_infinite_effect("c"));

    let _ = tick_all_effects(&mut container, 1);
    let outcome = tick_all_effects(&mut container, 2);

    assert_eq!(container.effects[0].remaining_turns, 0, "a should expire");
    assert_eq!(
        container.effects[1].remaining_turns, 3,
        "b should have 3 left"
    );
    assert_eq!(
        container.effects[2].remaining_turns,
        i64::MAX,
        "c should be MAX"
    );
    assert_eq!(outcome.expired.len(), 1);
    assert!(outcome.expired.contains(&"a".to_string()));
}

#[test]
fn tick_and_expire_merges_tick_and_expire() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_001", 1));
    push_active(&mut container, make_periodic_effect("dot_001", 3, 2));

    let outcome1 = tick_and_expire(&mut container, 1);
    assert!(
        outcome1.expired.contains(&"dur_001".to_string()),
        "dur_001 should expire after 1 turn"
    );

    assert_eq!(
        container.effects[0].stage,
        EffectStage::Removed,
        "expired effect should be Removed"
    );

    let outcome2 = tick_and_expire(&mut container, 2);
    assert!(
        outcome2.ticked.contains(&"dot_001".to_string()),
        "dot_001 should tick at interval 2"
    );
}

#[test]
fn expire_cleans_expiring_effects() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_001", 1));

    let _ = tick_all_effects(&mut container, 1);
    assert_eq!(container.effects[0].stage, EffectStage::Expiring);

    let expired = expire_all_effects(&mut container);
    assert!(expired.contains(&"dur_001".to_string()));
    assert_eq!(container.effects[0].stage, EffectStage::Removed);
}

#[test]
fn resume_all_effects_unpauses_paused_active_effects() {
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("paused_001", 3));
    container.effects[0].paused = true;

    resume_all_effects(&mut container);

    assert!(!container.effects[0].paused);
}
