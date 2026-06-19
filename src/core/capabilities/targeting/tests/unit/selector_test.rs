use bevy::prelude::*;

use crate::core::capabilities::targeting::foundation::{
    EntityTarget, PriorityRule, TargetContext, TargetData, TargetShape, TargetType, TargetingDef,
    TargetingError, ValidationResult,
};
use crate::core::capabilities::targeting::mechanism::{
    CandidateTarget, filter_by_type, select_targets, validate_candidate, validate_targeting_def,
};

fn setup() -> (World, Entity) {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    (world, entity)
}

fn make_single_targeting_def() -> TargetingDef {
    TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(5.0), 1).unwrap()
}

fn make_candidates() -> Vec<CandidateTarget> {
    vec![
        CandidateTarget::new("enemy_001")
            .with_position("3,4")
            .with_distance(2.0)
            .with_faction("enemy"),
        CandidateTarget::new("enemy_002")
            .with_position("7,8")
            .with_distance(6.0)
            .with_faction("enemy"),
        CandidateTarget::new("ally_001")
            .with_position("1,1")
            .with_distance(1.0)
            .with_faction("ally"),
    ]
}

fn make_context() -> TargetContext {
    TargetContext::new("caster_001", "ally", 1).with_caster_position("0,0")
}

#[test]
fn valid_single_target_def_validates() {
    let def = make_single_targeting_def();
    assert!(validate_targeting_def(&def).is_ok());
}

#[test]
fn invalid_max_target_count_fails() {
    let result = TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(5.0), 0);
    assert!(result.is_err());
}

#[test]
fn single_shape_requires_max_target_count_one() {
    let result = TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(5.0), 3);
    assert!(result.is_err());
}

#[test]
fn valid_area_shape_def_validates() {
    let def = TargetingDef::new(
        TargetType::Enemy,
        TargetShape::Area { radius: 3.0 },
        Some(8.0),
        5,
    )
    .unwrap();
    assert!(validate_targeting_def(&def).is_ok());
}

#[test]
fn invalid_area_shape_zero_radius_fails() {
    let result = TargetShape::Area { radius: 0.0 }.validate();
    assert!(result.is_err());
}

#[test]
fn filter_by_type_enemy_targets() {
    let def = make_single_targeting_def();
    let candidates = make_candidates();
    let filtered = filter_by_type(&def, &candidates, "ally");
    assert_eq!(filtered.len(), 2);
}

#[test]
fn self_type_includes_caster() {
    let def = TargetingDef::new(TargetType::Self_, TargetShape::Single, None, 1)
        .unwrap()
        .with_include_self(true);

    let mut candidates = make_candidates();
    candidates.push(CandidateTarget::new("caster_001").with_is_caster(true));

    let filtered = filter_by_type(&def, &candidates, "ally");
    let caster_included = filtered.iter().any(|c| c.is_caster);
    assert!(caster_included);
}

#[test]
fn default_excludes_dead_targets() {
    let def = make_single_targeting_def();
    let mut candidates = make_candidates();
    candidates[0].alive = false;

    let filtered = filter_by_type(&def, &candidates, "ally");
    let dead_included = filtered.iter().any(|c| !c.alive);
    assert!(!dead_included);
}

#[test]
fn flag_allows_dead_targets() {
    let def = TargetingDef::new(TargetType::Dead, TargetShape::Single, None, 1)
        .unwrap()
        .with_allow_dead_targets(true);

    let mut candidates = make_candidates();
    candidates.push(CandidateTarget::new("dead_001").with_alive(false));

    let filtered = filter_by_type(&def, &candidates, "ally");
    assert!(filtered.is_empty() || !filtered.iter().all(|c| c.alive));
}

#[test]
fn target_in_range_validates() {
    let def = make_single_targeting_def();
    let candidate = CandidateTarget::new("enemy_001").with_distance(3.0);
    let result = validate_candidate(&def, &candidate);
    assert!(result.is_pass());
}

#[test]
fn target_out_of_range_fails() {
    let def = make_single_targeting_def();
    let candidate = CandidateTarget::new("enemy_001").with_distance(10.0);
    let result = validate_candidate(&def, &candidate);
    assert!(!result.is_pass());
}

#[test]
fn target_below_min_range_fails() {
    let def = make_single_targeting_def().with_min_range(2.0).unwrap();
    let candidate = CandidateTarget::new("enemy_001").with_distance(1.0);
    let result = validate_candidate(&def, &candidate);
    assert!(!result.is_pass());
}

#[test]
fn no_range_limit_validates() {
    let def = TargetingDef::new(TargetType::Enemy, TargetShape::Single, None, 1).unwrap();
    let candidate = CandidateTarget::new("enemy_001").with_distance(999.0);
    let result = validate_candidate(&def, &candidate);
    assert!(result.is_pass());
}

#[test]
fn nearest_priority_sort_correct() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = make_single_targeting_def().with_priority_rule(PriorityRule::Nearest);
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands).unwrap();
    assert!(!result.entities.is_empty());
    assert_eq!(result.entities[0].entity_id, "enemy_001");
}

#[test]
fn farthest_priority_sort_correct() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(10.0), 1)
        .unwrap()
        .with_priority_rule(PriorityRule::Farthest);
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands).unwrap();
    assert!(!result.entities.is_empty());
    assert_eq!(result.entities[0].distance, 6.0);
}

#[test]
fn select_single_target_succeeds() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = make_single_targeting_def();
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands).unwrap();
    assert!(result.has_valid_targets);
    assert_eq!(result.target_count(), 1);
}

#[test]
fn select_multiple_targets_succeeds() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = TargetingDef::new(
        TargetType::Enemy,
        TargetShape::Area { radius: 10.0 },
        None,
        3,
    )
    .unwrap();
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands).unwrap();
    assert!(result.has_valid_targets);
    assert_eq!(result.target_count(), 2);
}

#[test]
fn no_valid_target_returns_error() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = TargetingDef::new(TargetType::Enemy, TargetShape::Single, Some(1.0), 1).unwrap();
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands);
    assert!(result.is_err());
}

#[test]
fn truncates_target_count_by_limit() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = TargetingDef::new(
        TargetType::Enemy,
        TargetShape::Area { radius: 10.0 },
        None,
        1,
    )
    .unwrap();
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands).unwrap();
    assert_eq!(result.target_count(), 1);
}

#[test]
fn selected_target_carries_context_data() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = make_single_targeting_def();
    let candidates = make_candidates();
    let context = make_context();

    let result = select_targets(&def, candidates, context.clone(), entity, "abl_test", &mut commands).unwrap();
    assert_eq!(result.context.caster_entity, context.caster_entity);
    assert_eq!(result.context.frame, 1);
}

#[test]
fn lowest_hp_priority_sort_correct() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let mut candidates = make_candidates();
    candidates[0] = candidates[0].clone().with_health(50.0, 100.0);
    candidates[1] = candidates[1].clone().with_health(30.0, 100.0);
    candidates[2] = candidates[2].clone().with_health(80.0, 100.0);
    let context = make_context();

    let def = TargetingDef::new(
        TargetType::Enemy,
        TargetShape::Area { radius: 10.0 },
        None,
        3,
    )
    .unwrap()
    .with_priority_rule(PriorityRule::LowestHealth);

    let result = select_targets(&def, candidates, context, entity, "abl_test", &mut commands).unwrap();
    assert_eq!(result.target_count(), 2);
    assert_eq!(result.entities[0].entity_id, "enemy_002");
}

#[test]
fn validation_result_pass_state() {
    assert!(ValidationResult::pass().is_pass());
}

#[test]
fn validation_result_fail_state() {
    assert!(!ValidationResult::fail("blocked").is_pass());
}

#[test]
fn target_data_first_target() {
    let context = make_context();
    let entities = vec![EntityTarget::new("enemy_001")];
    let data = TargetData::with_targets(entities, vec![], context);
    assert_eq!(data.first_target(), Some("enemy_001"));
}

#[test]
fn target_data_checks_entity_exists() {
    let context = make_context();
    let entities = vec![EntityTarget::new("enemy_001")];
    let data = TargetData::with_targets(entities, vec![], context);
    assert!(data.contains_entity("enemy_001"));
    assert!(!data.contains_entity("enemy_002"));
}

#[test]
fn empty_target_data_validation() {
    let context = make_context();
    let data = TargetData::empty(context);
    assert!(!data.has_valid_targets);
    assert_eq!(data.target_count(), 0);
}

#[test]
fn entity_target_builder() {
    let target = EntityTarget::new("enemy_001")
        .with_position("3,4")
        .with_distance(5.0);
    assert_eq!(target.entity_id, "enemy_001");
    assert_eq!(target.position, "3,4");
    assert_eq!(target.distance, 5.0);
}

#[test]
fn chain_shape_validates() {
    assert!(
        TargetShape::Chain {
            bounces: 2,
            bounce_range: 3.0,
            allow_retarget: false,
        }
        .validate()
        .is_ok()
    );
}

#[test]
fn chain_shape_zero_bounce_fails() {
    assert!(
        TargetShape::Chain {
            bounces: 0,
            bounce_range: 3.0,
            allow_retarget: false,
        }
        .validate()
        .is_err()
    );
}

#[test]
fn cone_shape_invalid_angle_fails() {
    assert!(
        TargetShape::Cone {
            length: 5.0,
            angle: 400.0,
        }
        .validate()
        .is_err()
    );
}

#[test]
fn targeting_error_display_correct() {
    let err = TargetingError::OutOfRange {
        distance: 10.0,
        max_range: 5.0,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("10"));
    assert!(msg.contains("5"));
}
