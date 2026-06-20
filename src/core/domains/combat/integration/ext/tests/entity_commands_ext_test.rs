//! Tests for EntityCommandsExt.

use bevy::prelude::*;

use crate::core::domains::combat::integration::ext::EntityCommandsExt;

#[test]
fn add_buff_does_not_panic() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut entity = commands.spawn_empty();

    // Should not panic as it's a stub that just logs.
    entity.add_buff("test_buff");
}

#[test]
fn heal_does_not_panic() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut entity = commands.spawn_empty();

    // Should not panic as it's a stub that just logs.
    entity.heal(50);
}

#[test]
fn kill_does_not_panic() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut entity = commands.spawn_empty();

    // Should not panic as it's a stub that just logs.
    entity.kill();
}
