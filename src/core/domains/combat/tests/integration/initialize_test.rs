use bevy::prelude::*;

use crate::core::domains::combat::integration::turn::mark_unit_dead;
use crate::core::domains::combat::components::{
    ActionPoints, CombatParticipant, TeamId, TurnEntry, TurnQueue,
};
use crate::core::domains::combat::systems::turn_systems::initialize_turn_order;

fn entity(id: u32) -> Entity {
    Entity::from_raw_u32(id).unwrap()
}

#[test]
fn initialize_turn_order_creates_components_and_turn_queue() {
    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();
    let e3 = world.spawn_empty().id();

    let team_a = TeamId::new("player");
    let team_b = TeamId::new("enemy");

    let entries = vec![
        (e1, team_a.clone(), 20),
        (e2, team_a.clone(), 15),
        (e3, team_b, 10),
    ];

    initialize_turn_order(&mut world.commands(), entries, 6.0);
    world.flush();

    // TurnQueue is inserted as a Resource
    let queue = world
        .get_resource::<TurnQueue>()
        .expect("TurnQueue should exist");
    assert_eq!(queue.len(), 3);
    assert_eq!(queue.round_number(), 1);
    assert_eq!(queue.current_index(), 0);

    // TurnEntry order matches input initiative
    let expected_initiatives = [20, 15, 10];
    for (i, &initiative) in expected_initiatives.iter().enumerate() {
        let entry = queue.entries().get(i).expect("entry should exist");
        assert_eq!(entry.initiative, initiative);
    }

    // ActionPoints on each entity
    for e in &[e1, e2, e3] {
        let ap = world
            .entity(*e)
            .get::<ActionPoints>()
            .expect("ActionPoints should exist");
        assert!(ap.standard_action);
        assert!((ap.max_movement - 6.0).abs() < f32::EPSILON);
    }

    // CombatParticipant on each entity
    let participants: Vec<&CombatParticipant> =
        world.query::<&CombatParticipant>().iter(&world).collect();
    assert_eq!(participants.len(), 3);
    for p in &participants {
        assert!(p.is_alive, "all participants start alive");
    }
}

#[test]
fn mark_unit_dead_sets_is_alive_false() {
    let mut participant = CombatParticipant::alive(TeamId::new("test"));
    assert!(participant.is_alive);

    mark_unit_dead(&mut participant);
    assert!(
        !participant.is_alive,
        "unit should be dead after mark_unit_dead"
    );
}

#[test]
fn turn_queue_advance_with_initialize_turn_order() {
    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();

    let team = TeamId::new("player");
    let entries = vec![(e1, team.clone(), 20), (e2, team, 10)];

    initialize_turn_order(&mut world.commands(), entries, 6.0);
    world.flush();

    let queue = world
        .get_resource::<TurnQueue>()
        .expect("TurnQueue should exist");
    assert_eq!(queue.current_index(), 0);

    // Simulate advance
    let mut q = TurnQueue::new(
        queue
            .entries()
            .iter()
            .map(|e| TurnEntry::new(e.entity, e.team_id.clone(), e.initiative))
            .collect(),
    );
    q.advance();
    assert_eq!(q.current_index(), 1);
}
