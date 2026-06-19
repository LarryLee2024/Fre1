use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::condition::foundation::{
    ComparisonOp, Condition, ConditionContext, CustomCondition, CustomConditionId,
    TagRequirementMode,
};
use crate::core::capabilities::condition::mechanism::{check_immunity, evaluate};

fn assert_passed(cond: &Condition, ctx: &ConditionContext, entity: Entity, commands: &mut Commands) {
    assert!(evaluate(cond, ctx, entity, commands).is_passed());
}

fn assert_failed(cond: &Condition, ctx: &ConditionContext, entity: Entity, commands: &mut Commands) {
    assert!(!evaluate(cond, ctx, entity, commands).is_passed());
}

#[test]
fn has_passes_when_tag_present() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into(), "Status.Stunned".into()]);
    let cond = Condition::TagRequirement {
        mode: TagRequirementMode::Has,
        tag_id: "Immune.Fire".into(),
    };
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn has_fails_when_tag_absent() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
    let cond = Condition::TagRequirement {
        mode: TagRequirementMode::Has,
        tag_id: "Immune.Ice".into(),
    };
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn not_passes_when_tag_absent() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
    let cond = Condition::TagRequirement {
        mode: TagRequirementMode::Not,
        tag_id: "Immune.Ice".into(),
    };
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn not_fails_when_tag_present() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
    let cond = Condition::TagRequirement {
        mode: TagRequirementMode::Not,
        tag_id: "Immune.Fire".into(),
    };
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn tag_fails_when_context_missing() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::empty();
    let cond = Condition::TagRequirement {
        mode: TagRequirementMode::Has,
        tag_id: "Immune.Fire".into(),
    };
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn attr_greater_than_threshold_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 18.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::AttributeCheck {
        attribute_id: "str".into(),
        operator: ComparisonOp::GreaterOrEqual,
        threshold: 15.0,
    };
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn attr_below_threshold_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 12.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::AttributeCheck {
        attribute_id: "str".into(),
        operator: ComparisonOp::GreaterOrEqual,
        threshold: 15.0,
    };
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn attr_missing_check_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_attributes(HashMap::new());
    let cond = Condition::AttributeCheck {
        attribute_id: "str".into(),
        operator: ComparisonOp::GreaterOrEqual,
        threshold: 15.0,
    };
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn resource_sufficient_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("mana".into(), 50.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::ResourceCheck {
        resource_id: "mana".into(),
        required_amount: 30.0,
    };
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn resource_insufficient_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("mana".into(), 20.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::ResourceCheck {
        resource_id: "mana".into(),
        required_amount: 30.0,
    };
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn resource_exact_amount_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("mana".into(), 30.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::ResourceCheck {
        resource_id: "mana".into(),
        required_amount: 30.0,
    };
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn and_all_conditions_pass() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 18.0);
    attrs.insert("dex".into(), 14.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::And(vec![
        Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        },
        Condition::AttributeCheck {
            attribute_id: "dex".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 12.0,
        },
    ]);
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn and_one_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 18.0);
    attrs.insert("dex".into(), 10.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::And(vec![
        Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        },
        Condition::AttributeCheck {
            attribute_id: "dex".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 12.0,
        },
    ]);
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn empty_and_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::empty();
    let cond = Condition::And(vec![]);
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn or_one_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 10.0);
    attrs.insert("dex".into(), 14.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::Or(vec![
        Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        },
        Condition::AttributeCheck {
            attribute_id: "dex".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 8.0,
        },
    ]);
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn or_all_fail() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 10.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::Or(vec![
        Condition::AttributeCheck {
            attribute_id: "str".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 15.0,
        },
        Condition::AttributeCheck {
            attribute_id: "dex".into(),
            operator: ComparisonOp::GreaterOrEqual,
            threshold: 12.0,
        },
    ]);
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn empty_or_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::empty();
    let cond = Condition::Or(vec![]);
    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn not_inverts_pass() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 18.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::Not(Box::new(Condition::AttributeCheck {
        attribute_id: "str".into(),
        operator: ComparisonOp::LessThan,
        threshold: 15.0,
    }));
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn not_inverts_still_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut attrs = HashMap::new();
    attrs.insert("str".into(), 10.0);
    let ctx = ConditionContext::with_attributes(attrs);
    let cond = Condition::Not(Box::new(Condition::AttributeCheck {
        attribute_id: "str".into(),
        operator: ComparisonOp::GreaterOrEqual,
        threshold: 15.0,
    }));
    assert_passed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn nested_and_or_combination_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = {
        let mut attrs = HashMap::new();
        attrs.insert("str".into(), 10.0);
        attrs.insert("dex".into(), 14.0);
        ConditionContext {
            tag_ids: Some(vec![]),
            attribute_values: attrs,
        }
    };

    let cond = Condition::Or(vec![
        Condition::And(vec![
            Condition::AttributeCheck {
                attribute_id: "str".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 15.0,
            },
            Condition::AttributeCheck {
                attribute_id: "dex".into(),
                operator: ComparisonOp::GreaterOrEqual,
                threshold: 12.0,
            },
        ]),
        Condition::TagRequirement {
            mode: TagRequirementMode::Has,
            tag_id: "Warrior".into(),
        },
    ]);

    assert_failed(&cond, &ctx, entity, &mut commands);
}

#[test]
fn no_immunity_check_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_tags(vec![]);
    assert!(check_immunity(&ctx, "Fire", entity, &mut commands).is_passed());
}

#[test]
fn immunity_check_fails() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::with_tags(vec!["Immune.Fire".into()]);
    assert!(!check_immunity(&ctx, "Fire", entity, &mut commands).is_passed());
}

#[test]
fn equal_comparison_correct() {
    assert!(ComparisonOp::Equal.evaluate(10.0, 10.0));
    assert!(!ComparisonOp::Equal.evaluate(10.0, 11.0));
}

#[test]
fn greater_than_comparison_correct() {
    assert!(ComparisonOp::GreaterThan.evaluate(10.0, 5.0));
    assert!(!ComparisonOp::GreaterThan.evaluate(5.0, 10.0));
    assert!(!ComparisonOp::GreaterThan.evaluate(5.0, 5.0));
}

#[test]
fn less_than_comparison_correct() {
    assert!(ComparisonOp::LessThan.evaluate(5.0, 10.0));
    assert!(!ComparisonOp::LessThan.evaluate(10.0, 5.0));
    assert!(!ComparisonOp::LessThan.evaluate(5.0, 5.0));
}

#[test]
fn custom_default_returns_failure() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let ctx = ConditionContext::empty();
    let cond = Condition::Custom(CustomCondition::new(CustomConditionId(42)));
    assert_failed(&cond, &ctx, entity, &mut commands);
}
