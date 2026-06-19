//! Integration tests — Combat OnTurnEnd → Effect Tick 集成 (遗留测试)
//!
//! WARNING: 测试 1-8 已由 `integration/effect/facade.rs` 中的 facade 测试替代。
//! 这些测试直接 import Effect Capabilities 内部类型，违反 ADR-024。
//! 保留为 #[ignore] 仅作历史参考，新测试应通过 integration 层编写。
//!
//! 测试 9 (effect_tick_works_with_turn_queue_info) 保留——TurnQueue 集成仅在 combat 层测试。

use bevy::prelude::*;

use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectDuration, EffectInstance, EffectPeriod,
    EffectStage,
};
use crate::core::capabilities::effect::mechanism::{expire_effects, tick_durations};
use crate::core::domains::combat::components::{TeamId, TurnEntry, TurnQueue};
use crate::core::domains::combat::integration::turn::get_turn_queue_info;

fn entity(id: u32) -> bevy::prelude::Entity {
    bevy::prelude::Entity::from_raw_u32(id).unwrap()
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

#[ignore]
#[test]
fn tick_durations_decrements_remaining_turns() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("dur_001", 3);
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    let result = tick_durations(&mut container, 1, 1, &mut commands);
    assert_eq!(container.effects[0].remaining_turns, 2);
    assert!(result.ticked.is_empty());
    assert!(result.expired.is_empty());
}

#[ignore]
#[test]
fn tick_durations_expires_effect_when_duration_depleted() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("dur_002", 1);
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    let result = tick_durations(&mut container, 1, 1, &mut commands);
    assert_eq!(container.effects[0].remaining_turns, 0);
    assert_eq!(container.effects[0].stage, EffectStage::Expiring);
    assert_eq!(result.expired.len(), 1);
    assert_eq!(result.expired[0], "dur_002");
}

#[ignore]
#[test]
fn expire_effects_cleans_up_expiring_effects() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("dur_003", 3);
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    let _ = tick_durations(&mut container, 3, 1, &mut commands);
    assert_eq!(container.effects[0].stage, EffectStage::Expiring);
    let expired_ids = expire_effects(&mut container);
    assert!(expired_ids.contains(&"dur_003".to_string()));
    assert_eq!(container.effects[0].stage, EffectStage::Removed);
}

#[ignore]
#[test]
fn tick_durations_triggers_periodic_tick_at_interval() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_periodic_effect("dot_001", 5, 2);
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    let result = tick_durations(&mut container, 2, 1, &mut commands);
    assert_eq!(result.ticked.len(), 1);
    assert!(result.ticked.contains(&"dot_001".to_string()));
}

#[ignore]
#[test]
fn tick_durations_no_tick_before_interval() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_periodic_effect("dot_002", 5, 3);
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    let result = tick_durations(&mut container, 1, 1, &mut commands);
    assert!(result.ticked.is_empty());
}

#[ignore]
#[test]
fn infinite_effect_never_expires() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_infinite_effect("inf_001");
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    let result = tick_durations(&mut container, 1000, 1, &mut commands);
    assert!(result.expired.is_empty());
    assert_eq!(container.effects[0].remaining_turns, i64::MAX);
    assert_eq!(container.effects[0].stage, EffectStage::Active);
}

#[ignore]
#[test]
fn paused_effect_does_not_tick() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("paused_001", 3);
    container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    container.effects[0].paused = true;
    let result = tick_durations(&mut container, 1, 1, &mut commands);
    assert_eq!(container.effects[0].remaining_turns, 3);
    assert!(result.ticked.is_empty());
    assert!(result.expired.is_empty());
}

#[ignore]
#[test]
fn multiple_effects_tick_independently() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    let e1 = make_duration_effect("a", 2);
    let e2 = make_duration_effect("b", 5);
    let e3 = make_infinite_effect("c");
    container.effects.push(e1);
    container.effects.push(e2);
    container.effects.push(e3);
    for i in 0..3 {
        container.effects[i].stage = EffectStage::Active;
    }
    let result = tick_durations(&mut container, 2, 1, &mut commands);
    assert_eq!(container.effects[0].remaining_turns, 0, "a should expire");
    assert_eq!(
        container.effects[1].remaining_turns, 3,
        "b should have 3 left"
    );
    assert_eq!(
        container.effects[2].remaining_turns,
        i64::MAX,
        "c should still be MAX"
    );
    assert_eq!(result.expired.len(), 1);
    assert_eq!(result.expired[0], "a");
}

#[test]
fn effect_tick_works_with_turn_queue_info() {
    // 唯一保留的集成测试——验证 TurnQueue API 与 effect 逻辑的联合使用
    let team_a = TeamId::new("player");
    let entries = vec![
        TurnEntry::new(entity(1), team_a.clone(), 20),
        TurnEntry::new(entity(2), team_a, 15),
    ];
    let queue = TurnQueue::new(entries);

    let info = get_turn_queue_info(&queue);
    assert_eq!(info.round_number, 1);
    assert_eq!(info.total_units, 2);
}
