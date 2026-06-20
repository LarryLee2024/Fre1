//! Rule Engine 单元测试

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bevy::prelude::*;

    use crate::core::capabilities::condition::foundation::{
        ComparisonOp, Condition, ConditionContext,
    };
    use crate::core::capabilities::rule::foundation::{RuleDef, RuleEffect, RuleModifierOp};
    use crate::core::capabilities::rule::mechanism::engine::{
        evaluate_rules, evaluate_single_rule,
    };
    use crate::shared::localization_key::LocalizationKey;

    fn make_rule(id: &str, condition: Condition, effect: RuleEffect) -> RuleDef {
        RuleDef {
            id: id.to_string(),
            name_key: LocalizationKey::new(&format!("rule.{}.name", id)),
            desc_key: LocalizationKey::new(&format!("rule.{}.desc", id)),
            condition,
            effect,
            priority: 0,
            enabled: true,
            domain: None,
        }
    }

    fn make_context_with_attrs(attrs: HashMap<String, f32>) -> ConditionContext {
        ConditionContext::with_attributes(attrs)
    }

    #[test]
    fn evaluate_rules_returns_matching_effects() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut commands = world.commands();

        let rule = make_rule(
            "rule_001",
            Condition::AttributeCheck {
                attribute_id: "hp".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: 50.0,
            },
            RuleEffect::Modifier {
                target_attribute: "fire_resistance".to_string(),
                op: RuleModifierOp::Add,
                value: -20.0,
                priority: 50,
            },
        );

        let mut attrs = HashMap::new();
        attrs.insert("hp".to_string(), 30.0);
        let context = make_context_with_attrs(attrs);

        let matches = evaluate_rules(&[rule], &context, entity, &mut commands);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_id, "rule_001");
    }

    #[test]
    fn evaluate_rules_skips_non_matching() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut commands = world.commands();

        let rule = make_rule(
            "rule_001",
            Condition::AttributeCheck {
                attribute_id: "hp".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: 50.0,
            },
            RuleEffect::Modifier {
                target_attribute: "fire_resistance".to_string(),
                op: RuleModifierOp::Add,
                value: -20.0,
                priority: 50,
            },
        );

        let mut attrs = HashMap::new();
        attrs.insert("hp".to_string(), 80.0);
        let context = make_context_with_attrs(attrs);

        let matches = evaluate_rules(&[rule], &context, entity, &mut commands);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn evaluate_rules_skips_disabled_rules() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut commands = world.commands();

        let mut rule = make_rule(
            "rule_001",
            Condition::AttributeCheck {
                attribute_id: "hp".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: 50.0,
            },
            RuleEffect::Modifier {
                target_attribute: "fire_resistance".to_string(),
                op: RuleModifierOp::Add,
                value: -20.0,
                priority: 50,
            },
        );
        rule.enabled = false;

        let mut attrs = HashMap::new();
        attrs.insert("hp".to_string(), 30.0);
        let context = make_context_with_attrs(attrs);

        let matches = evaluate_rules(&[rule], &context, entity, &mut commands);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn evaluate_rules_sorted_by_priority() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut commands = world.commands();

        let mut rule_high = make_rule(
            "rule_high",
            Condition::AttributeCheck {
                attribute_id: "hp".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: 100.0,
            },
            RuleEffect::Modifier {
                target_attribute: "a".to_string(),
                op: RuleModifierOp::Add,
                value: 1.0,
                priority: 100,
            },
        );
        rule_high.priority = 100;

        let mut rule_low = make_rule(
            "rule_low",
            Condition::AttributeCheck {
                attribute_id: "hp".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: 100.0,
            },
            RuleEffect::Modifier {
                target_attribute: "b".to_string(),
                op: RuleModifierOp::Add,
                value: 2.0,
                priority: 10,
            },
        );
        rule_low.priority = 10;

        let mut attrs = HashMap::new();
        attrs.insert("hp".to_string(), 50.0);
        let context = make_context_with_attrs(attrs);

        let matches = evaluate_rules(&[rule_high, rule_low], &context, entity, &mut commands);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].rule_id, "rule_low");
        assert_eq!(matches[1].rule_id, "rule_high");
    }

    #[test]
    fn evaluate_single_rule_returns_condition_result() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut commands = world.commands();

        let rule = make_rule(
            "rule_001",
            Condition::AttributeCheck {
                attribute_id: "hp".to_string(),
                operator: ComparisonOp::LessThan,
                threshold: 50.0,
            },
            RuleEffect::Modifier {
                target_attribute: "fire_resistance".to_string(),
                op: RuleModifierOp::Add,
                value: -20.0,
                priority: 50,
            },
        );

        let mut attrs = HashMap::new();
        attrs.insert("hp".to_string(), 30.0);
        let context = make_context_with_attrs(attrs);

        let result = evaluate_single_rule(&rule, &context, entity, &mut commands);
        assert!(result.is_passed());
    }

    #[test]
    fn rule_def_definition_type_constants() {
        use crate::content::loading::DefinitionType;
        assert_eq!(<RuleDef as DefinitionType>::BUCKET_NAME, "rules");
        assert_eq!(<RuleDef as DefinitionType>::EXTENSION, "ron");
    }
}
