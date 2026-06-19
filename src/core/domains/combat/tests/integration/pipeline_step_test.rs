use bevy::ecs::system::SystemState;
use bevy::prelude::*;

use crate::core::domains::combat::components::{
    ActionPoints, CombatParticipant, Dead, TeamId, TurnEntry, TurnQueue,
};
use crate::core::domains::combat::pipeline::steps::{
    PhaseCheckResult, TurnEndResult, check_team_elimination, step_phase_check, step_turn_end,
    step_turn_start,
};

fn make_player_team() -> TeamId {
    TeamId::new("player")
}

fn make_enemy_team() -> TeamId {
    TeamId::new("enemy")
}

// ═══════════════════════════════════════════════════════════════════
// step_phase_check
// ═══════════════════════════════════════════════════════════════════

#[test]
fn phase_check_empty_queue_returns_idle() {
    let turn_queue = TurnQueue::new(vec![]);
    let mut world = World::new();

    let mut ss: SystemState<Query<&mut ActionPoints>> = SystemState::new(&mut world);
    let ap_query = ss.get_mut(&mut world);

    let result = step_phase_check(&turn_queue, &ap_query);
    assert_eq!(result, PhaseCheckResult::Idle);
}

#[test]
fn phase_check_unit_with_actions_returns_has_actions() {
    let mut world = World::new();
    let e = world.spawn(ActionPoints::new(6.0)).id();
    let turn_queue = TurnQueue::new(vec![TurnEntry::new(e, make_player_team(), 20)]);

    let mut ss: SystemState<Query<&mut ActionPoints>> = SystemState::new(&mut world);
    let ap_query = ss.get_mut(&mut world);

    let result = step_phase_check(&turn_queue, &ap_query);
    assert_eq!(result, PhaseCheckResult::HasActions);
}

#[test]
fn phase_check_unit_idle_when_no_actions_and_no_movement() {
    let mut world = World::new();
    let mut ap = ActionPoints::new(0.0);
    ap.use_standard_action();
    ap.use_bonus_action();
    let e = world.spawn(ap).id();
    let turn_queue = TurnQueue::new(vec![TurnEntry::new(e, make_player_team(), 20)]);

    let mut ss: SystemState<Query<&mut ActionPoints>> = SystemState::new(&mut world);
    let ap_query = ss.get_mut(&mut world);

    let result = step_phase_check(&turn_queue, &ap_query);
    assert_eq!(result, PhaseCheckResult::Idle);
}

#[test]
fn phase_check_unit_with_only_movement_is_not_idle() {
    let mut world = World::new();
    let mut ap = ActionPoints::new(6.0);
    ap.use_standard_action();
    ap.use_bonus_action();
    // movement is still 6.0 after using both actions
    let e = world.spawn(ap).id();
    let turn_queue = TurnQueue::new(vec![TurnEntry::new(e, make_player_team(), 20)]);

    let mut ss: SystemState<Query<&mut ActionPoints>> = SystemState::new(&mut world);
    let ap_query = ss.get_mut(&mut world);

    let result = step_phase_check(&turn_queue, &ap_query);
    assert_eq!(
        result,
        PhaseCheckResult::HasActions,
        "unit with movement should have actions"
    );
}

#[test]
fn phase_check_unit_without_ap_component_returns_idle() {
    let mut world = World::new();
    // entity spawned without ActionPoints
    let e = world.spawn_empty().id();
    let turn_queue = TurnQueue::new(vec![TurnEntry::new(e, make_player_team(), 20)]);

    let mut ss: SystemState<Query<&mut ActionPoints>> = SystemState::new(&mut world);
    let ap_query = ss.get_mut(&mut world);

    let result = step_phase_check(&turn_queue, &ap_query);
    assert_eq!(result, PhaseCheckResult::Idle);
}

// ═══════════════════════════════════════════════════════════════════
// step_turn_start
// ═══════════════════════════════════════════════════════════════════

#[test]
fn turn_start_resets_action_points() {
    let mut world = World::new();
    let mut ap = ActionPoints::new(6.0);
    ap.use_standard_action();
    ap.use_bonus_action();
    let e = world.spawn(ap).id();
    let turn_queue = TurnQueue::new(vec![TurnEntry::new(e, make_player_team(), 20)]);

    let mut ss: SystemState<(Commands, Query<&mut ActionPoints>)> = SystemState::new(&mut world);
    {
        let (mut commands, mut ap_query) = ss.get_mut(&mut world);
        step_turn_start(&mut commands, &turn_queue, &mut ap_query);
    }
    ss.apply(&mut world);

    // Verify AP was reset
    let ap = world
        .entity(e)
        .get::<ActionPoints>()
        .expect("should have ActionPoints");
    assert!(ap.standard_action, "standard action should be restored");
    assert!(ap.bonus_action, "bonus action should be restored");
    assert!(
        (ap.movement - 6.0).abs() < f32::EPSILON,
        "movement should be restored to max"
    );
}

#[test]
fn turn_start_does_not_panic_on_empty_queue() {
    let turn_queue = TurnQueue::new(vec![]);
    let mut world = World::new();

    let mut ss: SystemState<(Commands, Query<&mut ActionPoints>)> = SystemState::new(&mut world);
    {
        let (mut commands, mut ap_query) = ss.get_mut(&mut world);
        // Should not panic
        step_turn_start(&mut commands, &turn_queue, &mut ap_query);
    }
}

#[test]
fn turn_start_does_not_panic_on_missing_ap_component() {
    let mut world = World::new();
    let e = world.spawn_empty().id();
    let turn_queue = TurnQueue::new(vec![TurnEntry::new(e, make_player_team(), 20)]);

    let mut ss: SystemState<(Commands, Query<&mut ActionPoints>)> = SystemState::new(&mut world);
    {
        let (mut commands, mut ap_query) = ss.get_mut(&mut world);
        // Should not panic, just warn
        step_turn_start(&mut commands, &turn_queue, &mut ap_query);
    }
}

// ═══════════════════════════════════════════════════════════════════
// step_turn_end
// ═══════════════════════════════════════════════════════════════════

#[test]
fn turn_end_advances_to_next_unit() {
    let mut world = World::new();
    let e1 = world
        .spawn(CombatParticipant::alive(make_player_team()))
        .id();
    let e2 = world
        .spawn(CombatParticipant::alive(make_enemy_team()))
        .id();
    let entries = vec![
        TurnEntry::new(e1, make_player_team(), 20),
        TurnEntry::new(e2, make_enemy_team(), 10),
    ];
    let mut turn_queue = TurnQueue::new(entries);
    let initial_index = turn_queue.current_index();

    let mut ss: SystemState<(
        Commands,
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let result = {
        let (mut commands, combatant_query, dead_query) = ss.get_mut(&mut world);
        step_turn_end(
            &mut commands,
            &mut turn_queue,
            &combatant_query,
            &dead_query,
        )
    };
    ss.apply(&mut world);

    assert_eq!(result, TurnEndResult::Continue);
    assert_eq!(
        turn_queue.current_index(),
        initial_index + 1,
        "turn queue should advance by 1"
    );
}

#[test]
fn turn_end_empty_queue_returns_battle_over() {
    let mut world = World::new();
    let mut turn_queue = TurnQueue::new(vec![]);

    let mut ss: SystemState<(
        Commands,
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let result = {
        let (mut commands, combatant_query, dead_query) = ss.get_mut(&mut world);
        step_turn_end(
            &mut commands,
            &mut turn_queue,
            &combatant_query,
            &dead_query,
        )
    };

    assert_eq!(result, TurnEndResult::BattleOver);
}

#[test]
fn turn_end_triggers_team_switch_event() {
    let mut world = World::new();
    let e1 = world
        .spawn(CombatParticipant::alive(make_player_team()))
        .id();
    let e2 = world
        .spawn(CombatParticipant::alive(make_enemy_team()))
        .id();
    let entries = vec![
        TurnEntry::new(e1, make_player_team(), 20),
        TurnEntry::new(e2, make_enemy_team(), 10),
    ];
    let mut turn_queue = TurnQueue::new(entries);

    let mut ss: SystemState<(
        Commands,
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let result = {
        let (mut commands, combatant_query, dead_query) = ss.get_mut(&mut world);
        step_turn_end(
            &mut commands,
            &mut turn_queue,
            &combatant_query,
            &dead_query,
        )
    };
    ss.apply(&mut world);

    assert_eq!(result, TurnEndResult::Continue);
    // After advance from index 0 → 1, we switched from player to enemy
    assert!(turn_queue.just_changed_team());
}

// ═══════════════════════════════════════════════════════════════════
// check_team_elimination
// ═══════════════════════════════════════════════════════════════════

#[test]
fn team_elimination_all_teams_alive_returns_false() {
    let mut world = World::new();
    world.spawn(CombatParticipant::alive(make_player_team()));
    world.spawn(CombatParticipant::alive(make_enemy_team()));

    let mut ss: SystemState<(
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let (query, dead_query) = ss.get_mut(&mut world);

    assert!(
        !check_team_elimination(&query, &dead_query),
        "battle should continue when all teams alive"
    );
}

#[test]
fn team_elimination_one_team_wiped_returns_true() {
    let mut world = World::new();
    // Player team: 2 alive
    world.spawn(CombatParticipant::alive(make_player_team()));
    world.spawn(CombatParticipant::alive(make_player_team()));
    // Enemy team: both dead (spawn with Dead tag)
    world.spawn((CombatParticipant::alive(make_enemy_team()), Dead));
    world.spawn((CombatParticipant::alive(make_enemy_team()), Dead));

    let mut ss: SystemState<(
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let (query, dead_query) = ss.get_mut(&mut world);

    assert!(
        check_team_elimination(&query, &dead_query),
        "battle should end when one team is wiped"
    );
}

#[test]
fn team_elimination_all_dead_returns_true() {
    let mut world = World::new();
    world.spawn((CombatParticipant::alive(make_player_team()), Dead));
    world.spawn((CombatParticipant::alive(make_enemy_team()), Dead));

    let mut ss: SystemState<(
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let (query, dead_query) = ss.get_mut(&mut world);

    assert!(
        check_team_elimination(&query, &dead_query),
        "battle should end when all teams are dead"
    );
}

#[test]
fn team_elimination_single_team_alive_returns_true() {
    let mut world = World::new();
    world.spawn(CombatParticipant::alive(make_player_team()));
    world.spawn(CombatParticipant::alive(make_player_team()));

    let mut ss: SystemState<(
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let (query, dead_query) = ss.get_mut(&mut world);

    assert!(
        check_team_elimination(&query, &dead_query),
        "battle should end with only one team"
    );
}

#[test]
fn team_elimination_empty_query_returns_true() {
    let mut world = World::new();

    let mut ss: SystemState<(
        Query<&CombatParticipant>,
        Query<&CombatParticipant, With<Dead>>,
    )> = SystemState::new(&mut world);
    let (query, dead_query) = ss.get_mut(&mut world);

    assert!(
        check_team_elimination(&query, &dead_query),
        "empty battlefield means battle over"
    );
}
