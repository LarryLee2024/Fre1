use bevy::prelude::*;

use crate::core::capabilities::aggregator::foundation::{
    CalcPipeline, CalcStage, ModifierEntry, ModifierOp, PipelineError, default_stages,
};
use crate::core::capabilities::aggregator::mechanism::pipeline::execute_aggregation;

fn make_entry(op: ModifierOp, magnitude: f32, priority: u8, target: &str) -> ModifierEntry {
    ModifierEntry {
        op,
        magnitude,
        priority,
        target_attribute: target.to_string(),
    }
}

fn default_pipeline_for(attr: &str) -> CalcPipeline {
    CalcPipeline {
        attribute_id: attr.to_string(),
        enabled_stages: default_stages(),
        priority_ascending: true,
        clamp_override: None,
        cycle_detection: true,
    }
}

#[test]
fn base_value_unchanged_without_modifiers() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &[],
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    assert_eq!(result.final_value, 10.0);
    assert_eq!(result.base_value, 10.0);
    assert_eq!(result.participating_count, 0);
}

#[test]
fn single_add_modifier_applies() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![make_entry(ModifierOp::Add, 5.0, 50, "attr_000001")];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    assert_eq!(result.final_value, 15.0);
}

#[test]
fn multiple_add_modifiers_sum() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![
        make_entry(ModifierOp::Add, 3.0, 50, "attr_000001"),
        make_entry(ModifierOp::Add, 7.0, 60, "attr_000001"),
    ];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // 10 + (3 + 7) = 20
    assert_eq!(result.final_value, 20.0);
    assert_eq!(result.participating_count, 2);
}

#[test]
fn multiply_is_compound_not_cumulative() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![
        make_entry(ModifierOp::Multiply, 1.2, 50, "attr_000001"),
        make_entry(ModifierOp::Multiply, 1.3, 60, "attr_000001"),
    ];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // 10 * 1.2 * 1.3 = 15.6
    assert!((result.final_value - 15.6).abs() < 1e-5);
}

#[test]
fn override_takes_highest_priority() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![
        make_entry(ModifierOp::Override, 50.0, 80, "attr_000001"),
        make_entry(ModifierOp::Override, 99.0, 10, "attr_000001"),
    ];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    assert_eq!(result.final_value, 99.0);
    assert!(result.was_overridden);
}

#[test]
fn skip_override_when_none_present() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![make_entry(ModifierOp::Add, 5.0, 50, "attr_000001")];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // 10 + 5 = 15, no override
    assert_eq!(result.final_value, 15.0);
    assert!(!result.was_overridden);
}

#[test]
fn clamp_lower_bound_applies() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![make_entry(ModifierOp::Add, -50.0, 50, "attr_000001")];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // 10 - 50 = -40 -> clamped to 0
    assert_eq!(result.final_value, 0.0);
}

#[test]
fn clamp_upper_bound_applies() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![make_entry(ModifierOp::Add, 200.0, 50, "attr_000001")];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // 10 + 200 = 210 -> clamped to 100
    assert_eq!(result.final_value, 100.0);
}

#[test]
fn clamp_override_uses_custom_range() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut pipeline = default_pipeline_for("attr_000001");
    pipeline.clamp_override = Some((5.0, 50.0));
    let modifiers = vec![make_entry(ModifierOp::Add, 200.0, 50, "attr_000001")];
    let result =
        execute_aggregation("attr_000001", 10.0, &modifiers, &pipeline, 0.0, 100.0, 1, entity, &mut commands).unwrap();
    // clamp_override (5, 50) overrides min=0, max=100
    assert_eq!(result.final_value, 50.0);
}

#[test]
fn invalid_clamp_range_returns_error() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut pipeline = default_pipeline_for("attr_000001");
    pipeline.clamp_override = Some((100.0, 0.0));
    let result = execute_aggregation("attr_000001", 10.0, &[], &pipeline, 0.0, 100.0, 1, entity, &mut commands);
    assert!(matches!(
        result,
        Err(PipelineError::InvalidClampBounds { .. })
    ));
}

#[test]
fn unrelated_modifier_ignored() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![make_entry(ModifierOp::Add, 99.0, 50, "attr_000002")];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    assert_eq!(result.final_value, 10.0);
    assert_eq!(result.participating_count, 0);
}

#[test]
fn full_pipeline_executes_all_stages_in_order() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![
        make_entry(ModifierOp::Add, 10.0, 50, "attr_000001"),
        make_entry(ModifierOp::Multiply, 1.5, 50, "attr_000001"),
        make_entry(ModifierOp::Override, 30.0, 99, "attr_000001"),
    ];
    let result = execute_aggregation(
        "attr_000001",
        5.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // Add: 5 + 10 = 15
    // Multiply: 15 * 1.5 = 22.5
    // Override: 30 (priority 99, only one)
    // Clamp: 30 in [0, 100]
    assert_eq!(result.final_value, 30.0);
}

#[test]
fn stage_values_tracked() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &[],
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        42,
        entity,
        &mut commands,
    )
    .unwrap();
    assert_eq!(result.frame, 42);
    assert!(result.stage_values.contains_key(&CalcStage::Clamp));
}

#[test]
fn descending_priority_ordering() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut pipeline = default_pipeline_for("attr_000001");
    pipeline.priority_ascending = false;
    let modifiers = vec![
        make_entry(ModifierOp::Override, 10.0, 10, "attr_000001"),
        make_entry(ModifierOp::Override, 99.0, 99, "attr_000001"),
    ];
    let result =
        execute_aggregation("attr_000001", 5.0, &modifiers, &pipeline, 0.0, 100.0, 1, entity, &mut commands).unwrap();
    // descending = true: higher value = more priority -> 99 wins
    assert_eq!(result.final_value, 99.0);
}

#[test]
fn skip_multiply_stage_when_no_multiply_modifier() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let modifiers = vec![make_entry(ModifierOp::Add, 5.0, 50, "attr_000001")];
    let result = execute_aggregation(
        "attr_000001",
        10.0,
        &modifiers,
        &default_pipeline_for("attr_000001"),
        0.0,
        100.0,
        1,
        entity,
        &mut commands,
    )
    .unwrap();
    // Add: 15, Multiply skipped: 15, Override skipped: 15, Clamp: 15
    assert_eq!(result.final_value, 15.0);
}
