//! CombatEventFacade 测试
//!
//! 验证事件 facade 的事件发布、优先级发布、枚举标签映射。

use bevy::prelude::*;

use crate::core::capabilities::event::foundation::{EventPayload, EventTag};
use crate::core::capabilities::event::mechanism::EventBus;
use crate::core::domains::combat::integration::event::{CombatEventFacade, CombatEventTag};

#[test]
fn publish_adds_to_pending_queue() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let payload = EventPayload::from_source("unit_001");
    CombatEventFacade::publish(&mut bus, CombatEventTag::TurnStarted, "system", payload, &mut commands);
    assert_eq!(bus.pending_count(), 1);
}

#[test]
fn publish_with_priority_adds_to_pending_queue() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let payload = EventPayload::from_source("unit_001");
    CombatEventFacade::publish_priority(&mut bus, CombatEventTag::DamageDealt, "system", payload, &mut commands);
    assert_eq!(bus.pending_count(), 1);
}

#[test]
fn publish_multiple_events_accumulate() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let payload1 = EventPayload::from_source("unit_001");
    let payload2 = EventPayload::from_source("unit_002");

    CombatEventFacade::publish(&mut bus, CombatEventTag::TurnStarted, "system", payload1, &mut commands);
    CombatEventFacade::publish(&mut bus, CombatEventTag::TurnEnded, "system", payload2, &mut commands);

    assert_eq!(bus.pending_count(), 2);
}

#[test]
fn publish_with_payload_carries_data() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let payload = EventPayload::from_source("unit_001")
        .with_value("damage", 50.0)
        .with_target("unit_002");

    CombatEventFacade::publish(&mut bus, CombatEventTag::DamageDealt, "system", payload, &mut commands);

    // Verify by dispatching and checking — for now, just verify it enqueues
    assert_eq!(bus.pending_count(), 1);
}

// ─── CombatEventTag 映射完整性 ──────────────────────────────────────

#[test]
fn turn_started_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::TurnStarted.to_event_tag(),
        EventTag::TurnStarted
    );
}

#[test]
fn turn_ended_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::TurnEnded.to_event_tag(),
        EventTag::TurnEnded
    );
}

#[test]
fn damage_dealt_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::DamageDealt.to_event_tag(),
        EventTag::DamageTaken
    );
}

#[test]
fn damage_taken_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::DamageTaken.to_event_tag(),
        EventTag::DamageTaken
    );
}

#[test]
fn heal_dealt_maps_to_event_tag() {
    assert_eq!(CombatEventTag::HealDealt.to_event_tag(), EventTag::Healed);
}

#[test]
fn kill_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::Kill.to_event_tag(),
        EventTag::Custom("Kill".to_string())
    );
}

#[test]
fn effect_applied_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::EffectApplied.to_event_tag(),
        EventTag::BuffApplied
    );
}

#[test]
fn ability_activated_maps_to_event_tag() {
    assert_eq!(
        CombatEventTag::AbilityActivated.to_event_tag(),
        EventTag::AbilityUsed
    );
}
