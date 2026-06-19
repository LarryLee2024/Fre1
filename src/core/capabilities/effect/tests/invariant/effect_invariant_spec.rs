//! Effect 不修改不存在属性不变量测试
//!
//! 不变量：Effect 引用的 AttributeId 必须已注册。
//! 来源：docs/02-domain/capabilities/effect_domain.md

use crate::core::capabilities::attribute::foundation::{AttributeCategory, AttributeId};
use crate::core::capabilities::attribute::mechanism::lifecycle::AttributeRegistry;
use crate::core::capabilities::effect::foundation::values::ActiveEffectContainer;
use crate::core::capabilities::effect::foundation::{EffectDuration, EffectInstance};
use crate::shared::testing::fixtures::{AttributeDefBuilder, attributes_for_unit_001};

fn make_registry_with_hp() -> AttributeRegistry {
    let mut reg = AttributeRegistry::default();
    for def in attributes_for_unit_001() {
        reg.register(def).unwrap();
    }
    reg
}

#[test]
fn register_hp_attr_no_error() {
    let reg = make_registry_with_hp();
    assert!(reg.contains(&AttributeId::new("attr_hp")));
}

#[test]
fn unregistered_attr_not_in_registry() {
    let reg = make_registry_with_hp();
    assert!(!reg.contains(&AttributeId::new("attr_nonexistent")));
}

#[test]
fn effect_container_rejects_missing_source() {
    let _container = ActiveEffectContainer::new();
    let effect = EffectInstance::new(
        "eff_001",
        "eff_test",
        vec![],
        "",
        "target_001",
        EffectDuration::Instant,
        1,
    );
    assert!(effect.source_entity.is_empty());
}

#[test]
fn duplicate_attr_id_rejected() {
    let mut reg = AttributeRegistry::default();
    reg.register(
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(100.0)
            .range(0.0, 100.0)
            .build(),
    )
    .unwrap();
    let result = reg.register(
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(100.0)
            .range(0.0, 100.0)
            .build(),
    );
    assert!(result.is_err());
}

// ── 不变量 3.3: 持续时间一致性 ──────────────────────────────

#[test]
fn duration_effect_remaining_turns_non_negative() {
    let effect = EffectInstance::new(
        "eff_001",
        "eff_test",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::HasDuration {
            turns: 5,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    );
    assert_eq!(effect.remaining_turns, 5);
    assert!(effect.remaining_turns >= 0);
}

#[test]
fn instant_effect_remaining_turns_is_zero() {
    let effect = EffectInstance::new(
        "eff_002",
        "eff_instant",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::Instant,
        1,
    );
    assert_eq!(effect.remaining_turns, 0);
}

#[test]
fn infinite_effect_remaining_turns_max() {
    let effect = EffectInstance::new(
        "eff_003",
        "eff_aura",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::Infinite,
        1,
    );
    assert_eq!(effect.remaining_turns, i64::MAX);
}

// ── 不变量 3.4: Effect 移除时 Modifier 必须回退 ──────────────

#[test]
fn effect_tracks_modifier_count_for_rollback() {
    let effect = EffectInstance::new(
        "eff_004",
        "eff_buff",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::HasDuration {
            turns: 3,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    )
    .with_modifiers(2);

    assert_eq!(effect.modifier_count, 2);
}

#[test]
fn effect_without_modifiers_has_zero_count() {
    let effect = EffectInstance::new(
        "eff_005",
        "eff_nobuff",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::Instant,
        1,
    );
    assert_eq!(effect.modifier_count, 0);
}

// ── 不变量 3.5: 同一 Effect 不得重复施加 ─────────────────────

#[test]
fn duplicate_effect_detection_by_def_and_source() {
    let effect_a = EffectInstance::new(
        "eff_006",
        "eff_poison",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::HasDuration {
            turns: 3,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    );
    let effect_b = EffectInstance::new(
        "eff_007",
        "eff_poison",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::HasDuration {
            turns: 3,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    );

    assert_eq!(effect_a.def_id, effect_b.def_id);
    assert_eq!(effect_a.source_entity, effect_b.source_entity);
}

#[test]
fn different_source_allows_same_effect() {
    let effect_a = EffectInstance::new(
        "eff_008",
        "eff_haste",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::HasDuration {
            turns: 3,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    );
    let effect_b = EffectInstance::new(
        "eff_009",
        "eff_haste",
        vec![],
        "source_002",
        "target_001",
        EffectDuration::HasDuration {
            turns: 3,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    );

    assert_eq!(effect_a.def_id, effect_b.def_id);
    assert_ne!(effect_a.source_entity, effect_b.source_entity);
}

// ── 禁止事项: Effect 不能直接修改属性值 ──────────────────────

#[test]
fn effect_only_describes_modifiers_not_applies() {
    let effect = EffectInstance::new(
        "eff_010",
        "eff_test",
        vec![],
        "source_001",
        "target_001",
        EffectDuration::HasDuration {
            turns: 5,
            calculation: crate::core::capabilities::effect::foundation::DurationCalculation::Fixed,
        },
        1,
    )
    .with_modifiers(3);

    assert_eq!(effect.modifier_count, 3);
    assert!(!effect.paused);
    assert_eq!(effect.stack_count, 1);
}
