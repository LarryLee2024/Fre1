//! Tests for QueryExt stub implementation.
//!
//! These tests verify that QueryExt methods are callable from a system context.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use crate::core::domains::combat::integration::ext::QueryExt;

/// Marker component for test entities.
#[derive(Component)]
struct _Health(u32);

/// System that calls alive() on a Query -- verifies the trait compiles.
fn alive_system(query: Query<&_Health>) {
    let _alive: Vec<Entity> = query.alive().collect();
}

/// System that calls hostile_to() on a Query -- verifies the trait compiles.
fn hostile_system(query: Query<&_Health>) {
    let _hostile: Vec<Entity> = query.hostile_to("enemy").collect();
}

#[test]
fn alive_method_compiles_in_system() {
    let mut app = App::new();
    app.add_systems(Update, alive_system);
    app.update();

    // Stub always returns empty, so this just verifies no crash.
}

#[test]
fn hostile_to_method_compiles_in_system() {
    let mut app = App::new();
    app.add_systems(Update, hostile_system);
    app.update();

    // Stub always returns empty, so this just verifies no crash.
}

#[test]
fn both_filters_return_empty() {
    let mut app = App::new();
    // Spawn an entity to verify the stub doesn't interact with real data.
    app.world_mut().spawn(_Health(100));

    app.add_systems(Update, |query: Query<&_Health>| {
        let alive: Vec<_> = query.alive().collect();
        assert!(alive.is_empty(), "stub alive() should return empty iterator");

        let hostile: Vec<_> = query.hostile_to("faction").collect();
        assert!(hostile.is_empty(), "stub hostile_to() should return empty iterator");
    });
    app.update();
}
