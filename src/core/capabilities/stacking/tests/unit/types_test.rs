use crate::core::capabilities::stacking::foundation::error::StackingError;
use crate::core::capabilities::stacking::foundation::types::{
    OverflowBehavior, StackIdentity, StackMatchResult, StackingConfig, StackingDecision,
    StackingType,
};

#[test]
fn stacking_type_names_correct() {
    assert_eq!(StackingType::None.name(), "None");
    assert_eq!(StackingType::Aggregate.name(), "Aggregate");
    assert_eq!(StackingType::RefreshDuration.name(), "RefreshDuration");
    assert_eq!(StackingType::Replace.name(), "Replace");
}

#[test]
fn stacking_type_tracks_layers_correct() {
    assert!(!StackingType::None.tracks_layers());
    assert!(StackingType::Aggregate.tracks_layers());
    assert!(!StackingType::RefreshDuration.tracks_layers());
    assert!(!StackingType::Replace.tracks_layers());
}

#[test]
fn none_type_config_created_successfully() {
    let config = StackingConfig::none();
    assert_eq!(config.stacking_type, StackingType::None);
    assert_eq!(config.max_stacks, 1);
    assert!(!config.allow_cross_source);
}

#[test]
fn aggregate_type_config_created_successfully() {
    let config = StackingConfig::aggregate(5, false).unwrap();
    assert_eq!(config.stacking_type, StackingType::Aggregate);
    assert_eq!(config.max_stacks, 5);
    assert!(config.reapply_modifiers_on_stack);
}

#[test]
fn aggregate_type_config_low_stacks_rejected() {
    let result = StackingConfig::aggregate(1, false);
    assert!(result.is_err());
}

#[test]
fn refresh_type_config_created_successfully() {
    let config = StackingConfig::refresh();
    assert_eq!(config.stacking_type, StackingType::RefreshDuration);
    assert_eq!(config.max_stacks, 1);
}

#[test]
fn replace_type_config_created_successfully() {
    let config = StackingConfig::replace().unwrap();
    assert_eq!(config.stacking_type, StackingType::Replace);
    assert_eq!(config.max_stacks, 1);
}

#[test]
fn overflow_behavior_names_correct() {
    assert_eq!(OverflowBehavior::IgnoreNew.name(), "IgnoreNew");
    assert_eq!(OverflowBehavior::Replace.name(), "Replace");
}

#[test]
fn basic_stack_identity_created_successfully() {
    let id = StackIdentity::new("eff_poison", "caster_001");
    assert_eq!(id.effect_def_id, "eff_poison");
    assert_eq!(id.source_entity, "caster_001");
    assert!(id.source_ability.is_none());
}

#[test]
fn stack_identity_with_ability_created_successfully() {
    let id = StackIdentity::new("eff_poison", "caster_001").with_ability("abl_fireball");
    assert_eq!(id.source_ability, Some("abl_fireball".into()));
}

#[test]
fn stack_match_result_variants_unique() {
    assert_ne!(StackMatchResult::FullMatch, StackMatchResult::NoMatch);
    assert_ne!(StackMatchResult::CrossSource, StackMatchResult::GroupMatch);
}

#[test]
fn stacking_decision_structure_validated() {
    let reject = StackingDecision::Reject;
    assert!(matches!(reject, StackingDecision::Reject));

    let acc = StackingDecision::Accumulate {
        new_stack_count: 3,
        added_layers: 1,
    };
    if let StackingDecision::Accumulate {
        new_stack_count,
        added_layers,
    } = &acc
    {
        assert_eq!(*new_stack_count, 3);
        assert_eq!(*added_layers, 1);
    } else {
        panic!("expected Accumulate");
    }
}

#[test]
fn stacking_error_display_correct() {
    let err = StackingError::InvalidConfig {
        reason: "max_stacks < 2 for Aggregate".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("invalid stacking config"));
}

#[test]
fn config_default_is_none_type() {
    let config = StackingConfig::default();
    assert_eq!(config.stacking_type, StackingType::None);
}
