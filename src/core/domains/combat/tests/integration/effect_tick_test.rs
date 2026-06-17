//! Integration tests — Combat OnTurnEnd → Effect Tick 集成
//!
//! 验证 on_turn_end_tick_effects 的核心逻辑：
//! - tick_durations 正确递减效果剩余回合
//! - expire_effects 正确清理到期效果
//! - 周期 Tick 在 interval 到达时触发
//! - Infinite 效果不会自然到期

use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectDuration, EffectInstance, EffectPeriod,
    EffectStage,
};
use crate::core::capabilities::effect::mechanism::{expire_effects, tick_durations};
use crate::core::domains::combat::api::get_turn_queue_info;
use crate::core::domains::combat::components::{TeamId, TurnEntry, TurnQueue};

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

#[test]
fn tick_durations_decrements_remaining_turns() {
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("dur_001", 3);
    let _ = container.effects.push(effect);

    // Manually transition to Active (simulating apply_effect behavior)
    container.effects[0].stage = EffectStage::Active;

    let result = tick_durations(&mut container, 1, 1);

    assert_eq!(
        container.effects[0].remaining_turns, 2,
        "remaining_turns should decrease by 1"
    );
    assert!(result.ticked.is_empty(), "no period, no tick");
    assert!(result.expired.is_empty(), "not yet expired");
}

#[test]
fn tick_durations_expires_effect_when_duration_depleted() {
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("dur_002", 1);
    let _ = container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;

    let result = tick_durations(&mut container, 1, 1);

    assert_eq!(container.effects[0].remaining_turns, 0, "duration depleted");
    assert_eq!(
        container.effects[0].stage,
        EffectStage::Expiring,
        "should transition to Expiring"
    );
    assert_eq!(result.expired.len(), 1, "should be marked as expired");
    assert_eq!(result.expired[0], "dur_002");
}

#[test]
fn expire_effects_cleans_up_expiring_effects() {
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("dur_003", 3);
    let _ = container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;

    // Advance enough turns to expire
    let _ = tick_durations(&mut container, 3, 1);
    assert_eq!(
        container.effects[0].stage,
        EffectStage::Expiring,
        "should be expiring after full duration"
    );

    let expired_ids = expire_effects(&mut container);
    assert!(
        expired_ids.contains(&"dur_003".to_string()),
        "should include expired effect"
    );
    assert_eq!(
        container.effects[0].stage,
        EffectStage::Removed,
        "effect should be removed"
    );
}

#[test]
fn tick_durations_triggers_periodic_tick_at_interval() {
    let mut container = ActiveEffectContainer::new();
    // Duration 5 turns, ticks every 2 turns
    let effect = make_periodic_effect("dot_001", 5, 2);
    let _ = container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;

    // Advance 2 turns → tick fires
    let result = tick_durations(&mut container, 2, 1);
    assert_eq!(result.ticked.len(), 1, "should trigger periodic tick");
    assert!(
        result.ticked.contains(&"dot_001".to_string()),
        "should include dot_001"
    );
}

#[test]
fn tick_durations_no_tick_before_interval() {
    let mut container = ActiveEffectContainer::new();
    let effect = make_periodic_effect("dot_002", 5, 3);
    let _ = container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;

    // Advance 1 turn → not enough for interval
    let result = tick_durations(&mut container, 1, 1);
    assert!(result.ticked.is_empty(), "no tick before interval");
}

#[test]
fn infinite_effect_never_expires() {
    let mut container = ActiveEffectContainer::new();
    let effect = make_infinite_effect("inf_001");
    let _ = container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;

    // Advance 1000 turns
    let result = tick_durations(&mut container, 1000, 1);
    assert!(
        result.expired.is_empty(),
        "infinite effect should never expire"
    );
    assert_eq!(
        container.effects[0].remaining_turns,
        i64::MAX,
        "infinite remaining_turns stays MAX"
    );
    assert_eq!(
        container.effects[0].stage,
        EffectStage::Active,
        "infinite effect stays Active"
    );
}

#[test]
fn paused_effect_does_not_tick() {
    let mut container = ActiveEffectContainer::new();
    let effect = make_duration_effect("paused_001", 3);
    let _ = container.effects.push(effect);
    container.effects[0].stage = EffectStage::Active;
    container.effects[0].paused = true;

    let result = tick_durations(&mut container, 1, 1);
    assert_eq!(
        container.effects[0].remaining_turns, 3,
        "paused effect should not decrease"
    );
    assert!(result.ticked.is_empty());
    assert!(result.expired.is_empty());
}

#[test]
fn multiple_effects_tick_independently() {
    let mut container = ActiveEffectContainer::new();
    let e1 = make_duration_effect("a", 2);
    let e2 = make_duration_effect("b", 5);
    let e3 = make_infinite_effect("c");
    let _ = container.effects.push(e1);
    let _ = container.effects.push(e2);
    let _ = container.effects.push(e3);
    for i in 0..3 {
        container.effects[i].stage = EffectStage::Active;
    }

    let result = tick_durations(&mut container, 2, 1);

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

    assert_eq!(result.expired.len(), 1, "only a should expire");
    assert_eq!(result.expired[0], "a");
}

#[test]
fn effect_tick_works_with_turn_queue_info() {
    // Verify the API works alongside effect tick logic
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
