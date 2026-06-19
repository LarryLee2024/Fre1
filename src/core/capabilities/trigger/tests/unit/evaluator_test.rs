use bevy::prelude::*;

use crate::core::capabilities::trigger::foundation::{
    TriggerCondition, TriggerEntry, TriggerFrequency, TriggerParams, TriggerType,
};
use crate::core::capabilities::trigger::mechanism::{
    TriggerEvalResult, build_trigger_context, can_trigger, check_frequency_limit,
    reset_all_frequencies,
};

#[test]
fn trigger_type_match_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
    let result = can_trigger(&entry, &TriggerType::OnDamaged, None, entity, &mut commands);
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn trigger_type_mismatch_blocked() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
    let result = can_trigger(&entry, &TriggerType::OnHealed, None, entity, &mut commands);
    assert!(matches!(result, TriggerEvalResult::Blocked(_)));
}

#[test]
fn unlimited_frequency_always_allowed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
    entry.record_trigger();
    entry.record_trigger();
    entry.record_trigger();
    let result = can_trigger(&entry, &TriggerType::OnDamaged, None, entity, &mut commands);
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn limited_frequency_exceeded_blocked() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut entry =
        TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(2);
    entry.record_trigger();
    entry.record_trigger();
    let result = can_trigger(&entry, &TriggerType::OnDamaged, None, entity, &mut commands);
    assert!(matches!(result, TriggerEvalResult::Blocked(_)));
}

#[test]
fn limited_frequency_within_limit_allowed() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut entry =
        TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(3);
    entry.record_trigger();
    let result = can_trigger(&entry, &TriggerType::OnDamaged, None, entity, &mut commands);
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn frequency_reset_allows_retrigger() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut entry =
        TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(1);
    entry.record_trigger();
    entry.reset_turn_count();
    let result = can_trigger(&entry, &TriggerType::OnDamaged, None, entity, &mut commands);
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn condition_check_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001")
        .with_condition(TriggerCondition::with_condition("hp_below_30"));
    let result = can_trigger(
        &entry,
        &TriggerType::OnDamaged,
        Some(&|cond_id| cond_id == "hp_below_30"),
        entity,
        &mut commands,
    );
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn condition_check_blocks() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001")
        .with_condition(TriggerCondition::with_condition("hp_below_30"));
    let result = can_trigger(
        &entry,
        &TriggerType::OnDamaged,
        Some(&|cond_id| cond_id == "is_raging"),
        entity,
        &mut commands,
    );
    assert!(matches!(result, TriggerEvalResult::Blocked(_)));
}

#[test]
fn no_condition_always_passes() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let entry = TriggerEntry::new("trig_001", TriggerType::OnTurnStart, "abl_000001");
    let result = can_trigger(
        &entry,
        &TriggerType::OnTurnStart,
        None,
        entity,
        &mut commands,
    );
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn build_context_fields_correct() {
    let entry = TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001");
    let mut payload = TriggerParams::new();
    payload.insert("damage_amount".into(), "42".into());
    let ctx = build_trigger_context(&entry, "entity_001", payload);
    assert_eq!(ctx.trigger_id, "trig_001");
    assert_eq!(ctx.target_ability_def_id, "abl_000001");
    assert_eq!(ctx.source_entity, "entity_001");
    assert_eq!(ctx.payload.get("damage_amount").unwrap(), "42");
}

#[test]
fn frequency_normal_returns_none() {
    let freq = TriggerFrequency::limited(3);
    assert!(check_frequency_limit(&freq, "trig_001").is_none());
}

#[test]
fn frequency_exceeded_returns_message() {
    let mut freq = TriggerFrequency::limited(1);
    freq.record_trigger();
    let msg = check_frequency_limit(&freq, "trig_001");
    assert!(msg.is_some());
    assert!(msg.unwrap().contains("suppressed"));
}

#[test]
fn unlimited_frequency_check_passes() {
    let freq = TriggerFrequency::unlimited();
    assert!(check_frequency_limit(&freq, "trig_001").is_none());
}

#[test]
fn entry_builder_creates_valid_entry() {
    let entry = TriggerEntry::new("trig_001", TriggerType::OnTurnEnd, "abl_000001")
        .with_condition(TriggerCondition::with_condition("cond_001"))
        .with_frequency(1);
    assert_eq!(entry.id, "trig_001");
    assert_eq!(entry.trigger_type, TriggerType::OnTurnEnd);
    assert_eq!(entry.condition.condition_id.unwrap(), "cond_001");
    assert_eq!(entry.frequency.max_per_turn, 1);
}

#[test]
fn reset_all_frequencies_works_correctly() {
    let mut entries = vec![
        TriggerEntry::new("trig_001", TriggerType::OnDamaged, "abl_000001").with_frequency(3),
        TriggerEntry::new("trig_002", TriggerType::OnHealed, "abl_000002").with_frequency(1),
    ];
    entries[0].record_trigger();
    entries[0].record_trigger();
    entries[1].record_trigger();
    reset_all_frequencies(&mut entries);
    assert_eq!(entries[0].frequency.current_turn_count, 0);
    assert_eq!(entries[1].frequency.current_turn_count, 0);
}

#[test]
fn trigger_type_name() {
    assert_eq!(TriggerType::OnDamaged.name(), "OnDamaged");
    assert_eq!(TriggerType::OnCustom("test".into()).name(), "OnCustom");
}
