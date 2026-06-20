//! CombatTargetingFacade 测试

use bevy::prelude::*;

use crate::core::capabilities::targeting::foundation::error::TargetingError;
use crate::core::capabilities::targeting::mechanism::CandidateTarget;
use crate::core::domains::combat::integration::targeting::CombatTargetingFacade;

fn setup() -> (World, Entity) {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    (world, entity)
}

#[test]
fn single_target_def_is_valid() {
    let def = CombatTargetingFacade::single_target_def(Some(5.0));
    assert!(CombatTargetingFacade::validate_def(&def).is_ok());
    assert_eq!(def.max_targets, 1);
}

#[test]
fn select_targets_returns_error_on_empty_candidates() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = CombatTargetingFacade::single_target_def(Some(5.0));
    let context = CombatTargetingFacade::create_target_context("caster_1", "faction_a", 1);
    let result = CombatTargetingFacade::select_targets(
        &def,
        vec![],
        context,
        entity,
        "abl_test",
        &mut commands,
    );
    assert!(matches!(result, Err(TargetingError::NoValidTargets { .. })));
}

#[test]
fn select_targets_filters_by_faction() {
    let (mut _world, entity) = setup();
    let mut commands = _world.commands();
    let def = CombatTargetingFacade::single_target_def(Some(10.0));
    let candidates = vec![
        CandidateTarget::new("target_1")
            .with_faction("faction_a")
            .with_alive(true),
        CandidateTarget::new("target_2")
            .with_faction("faction_b")
            .with_alive(true),
    ];
    let context = CombatTargetingFacade::create_target_context("caster_1", "faction_a", 1);
    let result = CombatTargetingFacade::select_targets(
        &def,
        candidates,
        context,
        entity,
        "abl_test",
        &mut commands,
    );
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.target_count(), 1);
    assert_eq!(data.first_target(), Some("target_2"));
}
