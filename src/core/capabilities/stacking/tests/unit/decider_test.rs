use crate::core::capabilities::stacking::foundation::{
    OverflowBehavior, StackingConfig, StackingDecision, StackingType,
};
use crate::core::capabilities::stacking::mechanism::decider::{
    StackingSubject, decide_stacking, evaluate_stacking, match_identity, validate_config,
};

fn subject(id: &str, def_id: &str, source: &str, turns: i64, stack: u32) -> StackingSubject {
    StackingSubject::new(id, def_id, source, turns, stack)
}

#[test]
fn same_def_match_identity_succeeds() {
    assert!(match_identity("eff_poison", "eff_poison"));
}

#[test]
fn different_def_match_identity_fails() {
    assert!(!match_identity("eff_poison", "eff_haste"));
}

#[test]
fn none_type_stacking_decision_rejected() {
    let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
    let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
    let config = StackingConfig::none();

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(decision, StackingDecision::Reject);
}

#[test]
fn aggregate_type_stacking_accumulates() {
    let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
    let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
    let config = StackingConfig::aggregate(5, false).unwrap();

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(
        decision,
        StackingDecision::Accumulate {
            new_stack_count: 2,
            added_layers: 1,
        }
    );
}

#[test]
fn aggregate_type_stacking_at_capacity_rejected() {
    let existing = subject("existing", "eff_poison", "caster_001", 3, 5);
    let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
    let config = StackingConfig::aggregate(5, false).unwrap();

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(decision, StackingDecision::Reject);
}

#[test]
fn aggregate_overflow_behavior_refresh() {
    let existing = subject("existing", "eff_poison", "caster_001", 2, 5);
    let incoming = subject("incoming", "eff_poison", "caster_001", 5, 1);
    let config = StackingConfig {
        stacking_type: StackingType::Aggregate,
        max_stacks: 5,
        allow_cross_source: false,
        overflow_behavior: OverflowBehavior::Refresh,
        reapply_modifiers_on_stack: true,
    };

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(
        decision,
        StackingDecision::Refresh {
            refreshed_instance_id: "existing".into(),
            new_duration: 5,
        }
    );
}

#[test]
fn aggregate_overflow_behavior_replace() {
    let existing = subject("existing", "eff_poison", "caster_001", 2, 5);
    let incoming = subject("incoming", "eff_poison", "caster_001", 5, 1);
    let config = StackingConfig {
        stacking_type: StackingType::Aggregate,
        max_stacks: 5,
        allow_cross_source: false,
        overflow_behavior: OverflowBehavior::Replace,
        reapply_modifiers_on_stack: true,
    };

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(
        decision,
        StackingDecision::Replace {
            replaced_instance_id: "existing".into(),
        }
    );
}

#[test]
fn refresh_type_takes_larger_duration() {
    let existing = subject("existing", "eff_haste", "caster_001", 3, 1);
    let incoming = subject("incoming", "eff_haste", "caster_001", 5, 1);
    let config = StackingConfig::refresh();

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(
        decision,
        StackingDecision::Refresh {
            refreshed_instance_id: "existing".into(),
            new_duration: 5,
        }
    );
}

#[test]
fn refresh_type_keeps_longer_duration() {
    let existing = subject("existing", "eff_haste", "caster_001", 10, 1);
    let incoming = subject("incoming", "eff_haste", "caster_001", 3, 1);
    let config = StackingConfig::refresh();

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(
        decision,
        StackingDecision::Refresh {
            refreshed_instance_id: "existing".into(),
            new_duration: 10,
        }
    );
}

#[test]
fn replace_type_directly_replaces() {
    let existing = subject("existing", "eff_buff", "caster_001", 3, 1);
    let incoming = subject("incoming", "eff_buff", "caster_002", 5, 1);
    let config = StackingConfig::replace().unwrap();

    let decision = decide_stacking(&existing, &incoming, &config);
    assert_eq!(
        decision,
        StackingDecision::Replace {
            replaced_instance_id: "existing".into(),
        }
    );
}

#[test]
fn different_def_evaluate_stacking_returns_none() {
    let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
    let incoming = subject("incoming", "eff_haste", "caster_001", 5, 1);
    let config = StackingConfig::aggregate(5, false).unwrap();

    let result = evaluate_stacking(&existing, &incoming, &config);
    assert!(result.is_none());
}

#[test]
fn same_def_evaluate_stacking_returns_result() {
    let existing = subject("existing", "eff_poison", "caster_001", 3, 1);
    let incoming = subject("incoming", "eff_poison", "caster_001", 3, 1);
    let config = StackingConfig::aggregate(5, false).unwrap();

    let result = evaluate_stacking(&existing, &incoming, &config);
    assert!(result.is_some());
    let outcome = result.unwrap();
    assert_eq!(outcome.new_stack_count, 2);
    assert!(matches!(
        outcome.decision,
        StackingDecision::Accumulate { .. }
    ));
}

#[test]
fn replace_evaluate_stacking_returns_one_stack() {
    let existing = subject("existing", "eff_buff", "caster_001", 3, 3);
    let incoming = subject("incoming", "eff_buff", "caster_002", 5, 1);
    let config = StackingConfig::replace().unwrap();

    let result = evaluate_stacking(&existing, &incoming, &config);
    assert!(result.is_some());
    let outcome = result.unwrap();
    assert_eq!(outcome.new_stack_count, 1);
}

#[test]
fn none_type_config_validates() {
    assert!(validate_config(&StackingConfig::none()).is_ok());
}

#[test]
fn aggregate_type_config_validates() {
    assert!(validate_config(&StackingConfig::aggregate(5, false).unwrap()).is_ok());
}

#[test]
fn aggregate_type_config_min_stacks_fails() {
    let config = StackingConfig {
        stacking_type: StackingType::Aggregate,
        max_stacks: 1,
        allow_cross_source: false,
        overflow_behavior: OverflowBehavior::IgnoreNew,
        reapply_modifiers_on_stack: true,
    };
    assert!(validate_config(&config).is_err());
}

#[test]
fn replace_type_config_max_stacks_error_fails() {
    let config = StackingConfig {
        stacking_type: StackingType::Replace,
        max_stacks: 3,
        allow_cross_source: true,
        overflow_behavior: OverflowBehavior::Replace,
        reapply_modifiers_on_stack: true,
    };
    assert!(validate_config(&config).is_err());
}

#[test]
fn refresh_type_config_validates() {
    assert!(validate_config(&StackingConfig::refresh()).is_ok());
}

#[test]
fn zero_max_stacks_config_fails() {
    let config = StackingConfig {
        stacking_type: StackingType::None,
        max_stacks: 0,
        allow_cross_source: false,
        overflow_behavior: OverflowBehavior::IgnoreNew,
        reapply_modifiers_on_stack: false,
    };
    assert!(validate_config(&config).is_err());
}
