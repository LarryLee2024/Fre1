//! CombatAggregatorFacade 测试

use bevy::prelude::*;

use crate::core::capabilities::aggregator::foundation::ModifierOp;
use crate::core::domains::combat::integration::aggregator::CombatAggregatorFacade;

#[test]
fn default_pipeline_has_all_stages() {
    let pipeline = CombatAggregatorFacade::default_pipeline("phys_atk");
    assert_eq!(pipeline.attribute_id, "phys_atk");
    assert!(!pipeline.enabled_stages.is_empty());
}

#[test]
fn execute_default_aggregation_additive() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![
        CombatAggregatorFacade::create_modifier_entry(
            ModifierOp::Add,
            10.0,
            1u8,
            "phys_atk".to_string(),
        ),
        CombatAggregatorFacade::create_modifier_entry(
            ModifierOp::Add,
            5.0,
            1u8,
            "phys_atk".to_string(),
        ),
    ];
    let result =
        CombatAggregatorFacade::execute_default_aggregation("phys_atk", 100.0, &modifiers, 0, entity, &mut commands);
    assert!(result.is_ok());
    let agg = result.unwrap();
    assert!((agg.final_value - 115.0).abs() < 0.001);
}

#[test]
fn create_modifier_entry_has_correct_fields() {
    let entry = CombatAggregatorFacade::create_modifier_entry(
        ModifierOp::Multiply,
        1.5,
        2u8,
        "phys_atk".to_string(),
    );
    assert_eq!(entry.op, ModifierOp::Multiply);
    assert!((entry.magnitude - 1.5).abs() < 0.001);
    assert_eq!(entry.priority, 2);
}

#[test]
fn execute_aggregation_no_modifiers() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let result = CombatAggregatorFacade::execute_default_aggregation("speed", 50.0, &[], 0, entity, &mut commands);
    assert!(result.is_ok());
    let agg = result.unwrap();
    assert!((agg.final_value - 50.0).abs() < 0.001);
}
