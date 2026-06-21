//! 不变量测试 — Combat 集成层的 Effect 生命周期不变量。
//!
//! | 不变量 | 来源 | 描述 |
//! |--------|------|------|
//! | 3.3 | effect_domain.md | tick 后 remaining_turns 不得为负数 |
//! | 2 | effect_domain.md | 阶段转移：Active -> Expiring -> Removed（不可跳过） |

use bevy::prelude::*;

use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectDuration, EffectInstance, EffectStage,
};
use crate::core::capabilities::effect::mechanism::{apply_effect, expire_effects, tick_durations};

// -- Helpers ---------------------------------------------------------------

/// 创建 Duration 效果，使用唯一的 (instance_id, def_id, source) 避免重复排斥。
fn make_duration_effect(id: &str, def_id: &str, turns: u32, source: &str) -> EffectInstance {
    EffectInstance::new(
        id,
        def_id,
        vec![],
        source,
        "target_001",
        EffectDuration::HasDuration {
            turns,
            calculation: DurationCalculation::Fixed,
        },
        1,
    )
}

fn make_infinite_effect(id: &str, source: &str) -> EffectInstance {
    EffectInstance::new(
        id,
        "eff_aura",
        vec![],
        source,
        "target_001",
        EffectDuration::Infinite,
        1,
    )
}

fn apply_and_activate(
    container: &mut ActiveEffectContainer,
    effect: EffectInstance,
    commands: &mut Commands,
) {
    let _ = apply_effect(container, effect, None, commands);
}

// -- Invariants ------------------------------------------------------------

/// 不变量 3.3: remaining_turns must never be negative.
#[test]
fn remaining_turns_never_negative() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    apply_and_activate(
        &mut container,
        make_duration_effect("a", "eff_a", 3, "src_a"),
        &mut commands,
    );
    apply_and_activate(
        &mut container,
        make_duration_effect("b", "eff_b", 1, "src_b"),
        &mut commands,
    );
    apply_and_activate(
        &mut container,
        make_infinite_effect("c", "src_c"),
        &mut commands,
    );

    let _ = tick_durations(&mut container, 100, 1, &mut commands);

    for effect in &container.effects {
        assert!(
            effect.remaining_turns >= 0,
            "remaining_turns must never be negative, got {} for '{}'",
            effect.remaining_turns,
            effect.instance_id
        );
    }
}

/// 不变量: After tick_durations + expire_effects, no effect remains Expiring.
#[test]
fn expire_effects_clears_all_expiring() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    apply_and_activate(
        &mut container,
        make_duration_effect("a", "eff_a", 2, "src_a"),
        &mut commands,
    );
    apply_and_activate(
        &mut container,
        make_duration_effect("b", "eff_b", 5, "src_b"),
        &mut commands,
    );
    apply_and_activate(
        &mut container,
        make_infinite_effect("c", "src_c"),
        &mut commands,
    );

    let _ = tick_durations(&mut container, 3, 1, &mut commands);
    let _ = expire_effects(&mut container);

    for effect in &container.effects {
        assert_ne!(
            effect.stage,
            EffectStage::Expiring,
            "no effects should remain Expiring after expire_effects, '{}' is still Expiring",
            effect.instance_id
        );
    }

    assert_eq!(
        container.find_by_id("a").unwrap().stage,
        EffectStage::Removed
    );
    assert_eq!(
        container.find_by_id("b").unwrap().stage,
        EffectStage::Active
    );
    assert_eq!(
        container.find_by_id("c").unwrap().stage,
        EffectStage::Active
    );
}

/// 不变量: expire_effects only touches Expiring effects.
#[test]
fn expire_effects_only_touches_expiring() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    apply_and_activate(
        &mut container,
        make_duration_effect("a", "eff_a", 1, "src_a"),
        &mut commands,
    );
    apply_and_activate(
        &mut container,
        make_duration_effect("b", "eff_b", 10, "src_b"),
        &mut commands,
    );

    let _ = tick_durations(&mut container, 1, 1, &mut commands);
    assert_eq!(
        container.find_by_id("a").unwrap().stage,
        EffectStage::Expiring
    );
    assert_eq!(
        container.find_by_id("b").unwrap().stage,
        EffectStage::Active
    );

    let expired = expire_effects(&mut container);

    assert!(
        expired.contains(&"a".to_string()),
        "'a' should be in expired list"
    );
    assert_eq!(expired.len(), 1, "only 'a' should expire");
    assert_eq!(
        container.find_by_id("a").unwrap().stage,
        EffectStage::Removed
    );
    assert_eq!(
        container.find_by_id("b").unwrap().stage,
        EffectStage::Active
    );
}

/// 不变量: tick_durations must not regress or skip effect stages.
#[test]
fn tick_durations_does_not_regress_stage() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = ActiveEffectContainer::new();
    apply_and_activate(
        &mut container,
        make_duration_effect("a", "eff_a", 1, "src_a"),
        &mut commands,
    );

    let _ = tick_durations(&mut container, 1, 1, &mut commands);
    assert_eq!(
        container.find_by_id("a").unwrap().stage,
        EffectStage::Expiring
    );

    let _ = expire_effects(&mut container);
    assert_eq!(
        container.find_by_id("a").unwrap().stage,
        EffectStage::Removed
    );

    let _ = tick_durations(&mut container, 5, 1, &mut commands);
    assert_eq!(
        container.find_by_id("a").unwrap().stage,
        EffectStage::Removed,
        "further ticks must not revive Removed effects"
    );
}
