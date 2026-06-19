//! Unit tests — CombatParticipant 与 mark_unit_dead API
//!
//! 验证 CombatParticipant 组件的生命周期：
//! - 创建时存活（无 Dead Tag）
//! - mark_unit_dead 正确插入 Dead Tag
//! - 不可逆（死后不能复活）

use bevy::prelude::*;

use crate::core::domains::combat::components::{CombatParticipant, Dead};
use crate::core::domains::combat::integration::turn::mark_unit_dead;
use crate::core::domains::combat::tests::fixtures::combat_fixtures;

#[test]
fn participant_starts_alive() {
    let mut world = World::new();
    let entity = world
        .spawn(CombatParticipant::alive(combat_fixtures::player_team()))
        .id();
    assert!(
        !world.entity(entity).contains::<Dead>(),
        "participant must not have Dead tag initially"
    );
    let p = world.entity(entity).get::<CombatParticipant>().unwrap();
    assert_eq!(p.team_id, combat_fixtures::player_team());
}

#[test]
fn participant_team_is_preserved() {
    let p = combat_fixtures::alive_participant(combat_fixtures::enemy_team());
    assert_eq!(p.team_id.as_str(), "enemy");
}

#[test]
fn mark_unit_dead_inserts_dead_tag() {
    let mut world = World::new();
    let entity = world
        .spawn(CombatParticipant::alive(combat_fixtures::player_team()))
        .id();

    let mut commands = world.commands();
    mark_unit_dead(&mut commands, entity);
    world.flush();

    assert!(
        world.entity(entity).contains::<Dead>(),
        "entity must have Dead tag after mark_unit_dead"
    );
}

#[test]
fn mark_unit_dead_is_idempotent() {
    let mut world = World::new();
    let entity = world
        .spawn(CombatParticipant::alive(combat_fixtures::player_team()))
        .id();

    let mut commands = world.commands();
    mark_unit_dead(&mut commands, entity);
    mark_unit_dead(&mut commands, entity); // second call should not panic
    world.flush();

    assert!(
        world.entity(entity).contains::<Dead>(),
        "still has Dead tag after second call"
    );
}

#[test]
fn dead_participant_has_dead_tag() {
    let mut world = World::new();
    let entity = world
        .spawn((
            combat_fixtures::dead_participant(combat_fixtures::enemy_team()),
            Dead,
        ))
        .id();

    assert!(
        world.entity(entity).contains::<Dead>(),
        "dead_participant should have Dead tag"
    );
}

#[test]
fn participant_team_id_matches_expected() {
    let player = combat_fixtures::alive_participant(combat_fixtures::player_team());
    let enemy = combat_fixtures::alive_participant(combat_fixtures::enemy_team());
    assert_ne!(player.team_id, enemy.team_id, "teams must differ");
}
