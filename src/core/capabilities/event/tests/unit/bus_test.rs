use std::sync::Arc;

use bevy::prelude::*;

use crate::core::capabilities::event::foundation::{
    DispatchReport, EventPayload, EventTag, SubscriberEntry,
};
use crate::core::capabilities::event::mechanism::EventBus;

// ── Basic publish/dispatch ─────────────────────────────

#[test]
fn publish_and_dispatch_single_event() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();

    bus.publish(
        EventTag::UnitSpawned,
        "test_system",
        EventPayload::from_source("entity_001"),
        &mut commands,
    );

    assert_eq!(bus.pending_count(), 1);
}

#[test]
fn dispatch_empty_queue() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let report = bus.dispatch_pending(&mut commands);
    assert_eq!(report.total, 0);
    assert!(report.all_succeeded());
}

#[test]
fn subscriber_receives_matching_event() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let received = Arc::new(std::sync::Mutex::new(false));
    let r = received.clone();

    bus.subscribe(SubscriberEntry {
        id: "test_sub".into(),
        tags: vec![EventTag::UnitSpawned],
        handler: Arc::new(move |_payload| {
            *r.lock().unwrap() = true;
            Ok(())
        }),
    });

    bus.publish(
        EventTag::UnitSpawned,
        "test",
        EventPayload::from_source("entity_001"),
        &mut commands,
    );

    let report = bus.dispatch_pending(&mut commands);
    assert_eq!(report.delivered, 1);
    assert!(*received.lock().unwrap());
}

#[test]
fn subscriber_ignores_non_matching_tag() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let received = Arc::new(std::sync::Mutex::new(false));
    let r = received.clone();

    bus.subscribe(SubscriberEntry {
        id: "test_sub".into(),
        tags: vec![EventTag::UnitDied],
        handler: Arc::new(move |_payload| {
            *r.lock().unwrap() = true;
            Ok(())
        }),
    });

    bus.publish(
        EventTag::UnitSpawned,
        "test",
        EventPayload::from_source("entity_001"),
        &mut commands,
    );

    let report = bus.dispatch_pending(&mut commands);
    assert_eq!(report.delivered, 0);
    assert!(!*received.lock().unwrap());
}

// ── Multiple subscribers ───────────────────────────────

#[test]
fn all_subscribers_receive_event() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let count = Arc::new(std::sync::Mutex::new(0u32));
    let c1 = count.clone();
    let c2 = count.clone();

    bus.subscribe(SubscriberEntry {
        id: "sub_a".into(),
        tags: vec![EventTag::DamageDealt],
        handler: Arc::new(move |_payload| {
            *c1.lock().unwrap() += 1;
            Ok(())
        }),
    });

    bus.subscribe(SubscriberEntry {
        id: "sub_b".into(),
        tags: vec![EventTag::DamageDealt],
        handler: Arc::new(move |_payload| {
            *c2.lock().unwrap() += 1;
            Ok(())
        }),
    });

    bus.publish(
        EventTag::DamageDealt,
        "test",
        EventPayload::from_source("entity_001"),
        &mut commands,
    );

    let report = bus.dispatch_pending(&mut commands);
    assert_eq!(report.delivered, 2);
    assert_eq!(*count.lock().unwrap(), 2);
}

// ── Handler failure isolation ──────────────────────────

#[test]
fn handler_failure_does_not_affect_other_subscribers() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let received = Arc::new(std::sync::Mutex::new(false));
    let r = received.clone();

    bus.subscribe(SubscriberEntry {
        id: "failing_sub".into(),
        tags: vec![EventTag::DamageDealt],
        handler: Arc::new(move |_payload| Err("simulated failure".into())),
    });

    bus.subscribe(SubscriberEntry {
        id: "ok_sub".into(),
        tags: vec![EventTag::DamageDealt],
        handler: Arc::new(move |_payload| {
            *r.lock().unwrap() = true;
            Ok(())
        }),
    });

    bus.publish(
        EventTag::DamageDealt,
        "test",
        EventPayload::from_source("entity_001"),
        &mut commands,
    );

    let report = bus.dispatch_pending(&mut commands);
    assert_eq!(report.delivered, 1);
    assert_eq!(report.failed, 1);
    assert!(*received.lock().unwrap());
}

// ── Cycle detection ────────────────────────────────────

#[test]
fn cycle_detection_stops_at_limit() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();

    let bus_ptr = Arc::new(std::sync::Mutex::new(Vec::new()));
    let b = bus_ptr.clone();

    bus.subscribe(SubscriberEntry {
        id: "cyclic_sub".into(),
        tags: vec![EventTag::DamageDealt],
        handler: Arc::new(move |_payload| {
            b.lock().unwrap().push("called".to_string());
            Ok(())
        }),
    });

    // Manually simulate cycle: publish 6 events (exceeds limit of 5)
    for _ in 0..6 {
        bus.publish(
            EventTag::DamageDealt,
            "test",
            EventPayload::from_source("entity_001"),
            &mut commands,
        );
    }

    let report = bus.dispatch_pending(&mut commands);
    assert!(report.cycle_interrupted);
    assert!(report.total <= 5 * 1);
}

// ── Subscribe/unsubscribe ──────────────────────────────

#[test]
fn unsubscribe_removes_subscriber() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    bus.subscribe(SubscriberEntry {
        id: "test_sub".into(),
        tags: vec![EventTag::UnitSpawned],
        handler: Arc::new(move |_payload| Ok(())),
    });

    assert_eq!(bus.total_subscribers(), 1);
    bus.unsubscribe("test_sub");
    assert_eq!(bus.total_subscribers(), 0);
}

#[test]
fn unsubscribe_idempotent() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    bus.unsubscribe("nonexistent_sub");
    assert_eq!(bus.total_subscribers(), 0);
}

#[test]
fn resubscribe_overrides_old() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    let result = Arc::new(std::sync::Mutex::new(String::new()));
    let r1 = result.clone();
    let r2 = result.clone();

    // First registration
    bus.subscribe(SubscriberEntry {
        id: "test_sub".into(),
        tags: vec![EventTag::UnitSpawned],
        handler: Arc::new(move |_payload| {
            *r1.lock().unwrap() = "first".into();
            Ok(())
        }),
    });

    // Overwrite
    bus.subscribe(SubscriberEntry {
        id: "test_sub".into(),
        tags: vec![EventTag::UnitSpawned],
        handler: Arc::new(move |_payload| {
            *r2.lock().unwrap() = "second".into();
            Ok(())
        }),
    });

    bus.publish(
        EventTag::UnitSpawned,
        "test",
        EventPayload::from_source("entity_001"),
        &mut commands,
    );
    let report = bus.dispatch_pending(&mut commands);
    assert_eq!(report.delivered, 1);
    assert_eq!(*result.lock().unwrap(), "second");
}

// ── EventPayload builder ───────────────────────────────

#[test]
fn payload_builder_correct() {
    let payload = EventPayload::from_source("entity_001")
        .with_target("entity_002")
        .with_value("damage", 42.0)
        .with_data("element", "fire")
        .with_tag("crit");

    assert_eq!(payload.source_entity, "entity_001");
    assert_eq!(payload.target_entity.unwrap(), "entity_002");
    assert_eq!(payload.values.get("damage").unwrap(), &42.0);
    assert_eq!(payload.custom_data.get("element").unwrap(), "fire");
    assert!(payload.tags.contains(&"crit".to_string()));
}

// ── EventTag name ──────────────────────────────────────

#[test]
fn event_tag_name_correct() {
    assert_eq!(EventTag::DamageDealt.name(), "DamageDealt");
    assert_eq!(EventTag::Custom("test".into()).name(), "Custom");
}

// ── DispatchReport ─────────────────────────────────────

#[test]
fn report_all_successful() {
    let report = DispatchReport {
        total: 3,
        delivered: 3,
        failed: 0,
        errors: vec![],
        cycle_interrupted: false,
    };
    assert!(report.all_succeeded());
}

#[test]
fn report_with_error_fails() {
    let report = DispatchReport {
        total: 3,
        delivered: 2,
        failed: 1,
        errors: vec![("sub_001".into(), "error".into())],
        cycle_interrupted: false,
    };
    assert!(!report.all_succeeded());
}

#[test]
fn reset_cycle_counters() {
    let mut world = World::new();
    let _entity = world.spawn_empty().id();
    let mut commands = world.commands();
    let mut bus = EventBus::new();
    // TODO: cycle_counters 是私有字段，无法直接验证重置结果
    // 需要 @feature-developer 暴露 pub fn cycle_count(&self, tag: &EventTag) -> u32
    // 或 pub fn is_cycle_counters_empty(&self) -> bool
    bus.reset_cycle_counters();
    // 验证方法不 panic 即可
}
