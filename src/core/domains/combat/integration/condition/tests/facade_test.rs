use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::condition::foundation::ConditionContext;
use crate::core::domains::combat::integration::condition::CombatConditionFacade;

#[test]
fn immunity_check_passed_when_no_immune_tag() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let context = ConditionContext {
        tag_ids: Some(vec!["Fire".to_string()]),
        tag_bits: 0,
        tag_masks: None,
        attribute_values: HashMap::new(),
    };
    let result =
        CombatConditionFacade::check_effect_immunity(&context, "Fire", entity, &mut commands);
    assert!(result.is_passed());
}

#[test]
fn immunity_check_failed_when_immune_tag_present() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let context = ConditionContext {
        tag_ids: Some(vec!["Immune.Fire".to_string()]),
        tag_bits: 0,
        tag_masks: None,
        attribute_values: HashMap::new(),
    };
    let result =
        CombatConditionFacade::check_effect_immunity(&context, "Fire", entity, &mut commands);
    assert!(!result.is_passed());
}
