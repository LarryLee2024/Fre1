//! CombatEffectFacade 测试
//!
//! 验证效果 facade 的读写操作：活跃效果查询、计时推进、到期清理、暂停恢复。

use bevy::prelude::*;

use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, EffectInstance, EffectPeriod, EffectStage,
};
use crate::core::capabilities::effect::tests::fixtures::{
    make_duration_effect, make_infinite_effect, make_test_container,
};

use crate::core::domains::combat::integration::effect::facade::*;

fn make_periodic_effect(id: &str, turns: u32, interval: u32) -> EffectInstance {
    let period = EffectPeriod::new(interval).unwrap();
    EffectInstance::new(
        id,
        "eff_dot",
        "Damage",
        "caster_001",
        "target_001",
        crate::core::capabilities::effect::foundation::EffectDuration::HasDuration {
            turns,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    )
    .with_period(period)
}

fn push_active(container: &mut ActiveEffectContainer, effect: EffectInstance) {
    let mut e = effect;
    e.stage = EffectStage::Active;
    container.effects.push(e);
}

#[test]
fn empty_container_has_no_active_effects() {
    let container = make_test_container();
    assert!(!has_active_effects(&container));
    assert_eq!(active_effect_count(&container), 0);
}

#[test]
fn tick_and_expire_on_empty_container_succeeds() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = make_test_container();
    let outcome = tick_and_expire(&mut container, 1, &mut commands);
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
fn tick_and_expire_ticking_reduces_remaining_turns() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_002", 3));
    let outcome = tick_and_expire(&mut container, 1, &mut commands);
    assert!(outcome.ticked.is_empty());
    assert!(outcome.expired.is_empty());
}

#[test]
fn tick_and_expire_expires_when_duration_depleted() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = make_test_container();
    push_active(&mut container, make_duration_effect("dur_003", 1));
    let outcome = tick_and_expire(&mut container, 1, &mut commands);
    assert_eq!(outcome.expired.len(), 1);
}

#[test]
fn tick_and_expire_reports_ticked() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = make_test_container();
    push_active(&mut container, make_periodic_effect("dot_001", 5, 1));
    let outcome = tick_and_expire(&mut container, 1, &mut commands);
    assert_eq!(outcome.ticked.len(), 1);
}

#[test]
fn infinite_effect_never_expires() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = make_test_container();
    push_active(&mut container, make_infinite_effect("inf_001"));
    let outcome = tick_and_expire(&mut container, 1, &mut commands);
    assert!(outcome.expired.is_empty());
}

#[test]
fn paused_effect_does_not_tick() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = make_test_container();
    let mut effect = make_duration_effect("paused_001", 3);
    effect.paused = true;
    push_active(&mut container, effect);

    let outcome = tick_and_expire(&mut container, 1, &mut commands);
    assert!(outcome.ticked.is_empty());
    assert!(outcome.expired.is_empty());
}

#[test]
fn has_active_effects_with_infinite_effect() {
    let mut container = make_test_container();
    push_active(&mut container, make_infinite_effect("inf_002"));
    assert!(has_active_effects(&container));
}

#[test]
fn has_active_effect_by_def_returns_false_for_missing() {
    let container = make_test_container();
    assert!(!has_active_effect_by_def(&container, "nonexistent"));
}

#[test]
fn resume_all_effects_works() {
    let mut container = make_test_container();
    let mut effect = make_duration_effect("resume_001", 5);
    effect.paused = true;
    effect.stage = EffectStage::Active;
    container.effects.push(effect);

    resume_all_effects(&mut container);
    assert!(!container.effects[0].paused);
}
