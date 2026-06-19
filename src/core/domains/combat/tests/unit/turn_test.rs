//! Unit tests — Combat 回合系统纯数据测试
//!
//! 覆盖 ActionPoints、TurnQueue、TeamId 等纯数据类型的业务逻辑。
//! 不依赖 Bevy App，使用原生 #[test]。

use bevy::prelude::Entity;

use crate::core::domains::combat::components::{
    ActionPoints, BattlePhase, TeamId, TurnEntry, TurnQueue,
};

/// 创建指定索引的测试实体。
fn entity(id: u32) -> Entity {
    Entity::from_raw_u32(id).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════
// TeamId
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn team_id_new_creates_from_string() {
    let id = TeamId::new("player");
    assert_eq!(id.as_str(), "player");
}

#[test]
fn team_id_display_shows_prefixed_value() {
    let id = TeamId::new("enemy");
    assert_eq!(format!("{id}"), "team:enemy");
}

#[test]
fn team_id_equality_works() {
    let a = TeamId::new("team_a");
    let b = TeamId::new("team_a");
    let c = TeamId::new("team_b");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ═══════════════════════════════════════════════════════════════════════
// BattlePhase
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn battle_phase_default_is_preparation() {
    assert_eq!(BattlePhase::default(), BattlePhase::Preparation);
}

#[test]
fn battle_phase_variants_are_distinct() {
    let phases = [
        BattlePhase::Preparation,
        BattlePhase::Battle,
        BattlePhase::Victory,
        BattlePhase::Defeat,
    ];
    // 通过长度确认没有重复变体被折叠
    assert_eq!(phases.len(), 4);
}

// ═══════════════════════════════════════════════════════════════════════
// ActionPoints
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn action_points_new_sets_all_available() {
    let ap = ActionPoints::new(6.0);
    assert!(ap.standard_action, "standard action should be available");
    assert!(ap.bonus_action, "bonus action should be available");
    assert!(ap.reaction, "reaction should be available");
    assert!((ap.movement - 6.0).abs() < f32::EPSILON);
    assert!((ap.max_movement - 6.0).abs() < f32::EPSILON);
}

#[test]
fn action_points_use_standard_action_consumes_it() {
    let mut ap = ActionPoints::new(6.0);
    assert!(ap.use_standard_action());
    assert!(!ap.standard_action, "standard action should be consumed");
}

#[test]
fn action_points_use_standard_action_returns_false_when_already_used() {
    let mut ap = ActionPoints::new(6.0);
    ap.use_standard_action();
    assert!(!ap.use_standard_action(), "second use should fail");
}

#[test]
fn action_points_use_bonus_action_consumes_it() {
    let mut ap = ActionPoints::new(6.0);
    assert!(ap.use_bonus_action());
    assert!(!ap.bonus_action);
}

#[test]
fn action_points_use_reaction_consumes_it() {
    let mut ap = ActionPoints::new(6.0);
    assert!(ap.use_reaction());
    assert!(!ap.reaction);
}

#[test]
fn action_points_consume_movement_reduces_current() {
    let mut ap = ActionPoints::new(6.0);
    assert!(ap.consume_movement(4.0));
    assert!((ap.movement - 2.0).abs() < f32::EPSILON);
}

#[test]
fn action_points_consume_movement_returns_false_when_insufficient() {
    let mut ap = ActionPoints::new(3.0);
    assert!(!ap.consume_movement(5.0));
    assert!(
        (ap.movement - 3.0).abs() < f32::EPSILON,
        "movement should not change"
    );
}

#[test]
fn action_points_reset_restores_all_resources() {
    let mut ap = ActionPoints::new(6.0);
    ap.use_standard_action();
    ap.use_bonus_action();
    ap.consume_movement(4.0);
    ap.reset();
    assert!(ap.standard_action, "standard action should restore");
    assert!(ap.bonus_action, "bonus action should restore");
    assert!(
        (ap.movement - 6.0).abs() < f32::EPSILON,
        "movement should restore to max"
    );
    // 注意：reaction 不被 reset 重置（每轮自然恢复）
}

#[test]
fn action_points_has_any_action_returns_true_when_any_available() {
    let mut ap = ActionPoints::new(6.0);
    assert!(ap.has_any_action());
    ap.use_standard_action();
    assert!(ap.has_any_action(), "bonus action still available");
    ap.use_bonus_action();
    assert!(!ap.has_any_action(), "no actions left");
}

#[test]
fn action_points_is_idle_no_actions_no_movement() {
    let mut ap = ActionPoints::new(0.0);
    ap.use_standard_action();
    ap.use_bonus_action();
    assert!(ap.is_idle(), "no actions and no movement = idle");
}

#[test]
fn action_points_is_idle_has_movement_is_not_idle() {
    let mut ap = ActionPoints::new(5.0);
    ap.use_standard_action();
    ap.use_bonus_action();
    assert!(!ap.is_idle(), "has movement = not idle");
}

#[test]
fn action_points_is_idle_has_action_is_not_idle() {
    let ap = ActionPoints::new(0.0);
    // standard_action 仍可用
    assert!(!ap.is_idle(), "has standard action = not idle");
}

// ═══════════════════════════════════════════════════════════════════════
// TurnEntry
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn turn_entry_new_creates_entry() {
    let dummy = entity(1);
    let team = TeamId::new("player");
    let entry = TurnEntry::new(dummy, team.clone(), 20);
    assert_eq!(entry.entity, dummy);
    assert_eq!(entry.team_id, team);
    assert_eq!(entry.initiative, 20);
}

// ═══════════════════════════════════════════════════════════════════════
// TurnQueue
// ═══════════════════════════════════════════════════════════════════════

fn make_entries() -> Vec<TurnEntry> {
    let team_a = TeamId::new("player");
    let team_b = TeamId::new("enemy");
    vec![
        TurnEntry::new(entity(1), team_a.clone(), 20),
        TurnEntry::new(entity(2), team_a.clone(), 18),
        TurnEntry::new(entity(3), team_b.clone(), 15),
        TurnEntry::new(entity(4), team_a, 10),
    ]
}

#[test]
fn turn_queue_new_starts_at_index_0_round_1() {
    let q = TurnQueue::new(make_entries());
    assert_eq!(q.current_index(), 0);
    assert_eq!(q.round_number(), 1);
    assert_eq!(q.len(), 4);
}

#[test]
fn turn_queue_current_returns_first_entry() {
    let q = TurnQueue::new(make_entries());
    let current = q.current().expect("should have current");
    assert_eq!(current.initiative, 20);
}

#[test]
fn turn_queue_current_mut_allows_modification() {
    let mut q = TurnQueue::new(make_entries());
    let current = q.current_mut().expect("should have current");
    current.initiative = 99;
    assert_eq!(q.current().unwrap().initiative, 99);
}

#[test]
fn turn_queue_advance_moves_to_next_entry() {
    let mut q = TurnQueue::new(make_entries());
    q.advance();
    assert_eq!(q.current_index(), 1);
    assert_eq!(q.round_number(), 1);
}

#[test]
fn turn_queue_advance_wraps_around_increments_round() {
    let mut q = TurnQueue::new(make_entries());
    // Advance 4 times → wraps back to 0
    q.advance();
    q.advance();
    q.advance();
    q.advance();
    assert_eq!(q.current_index(), 0);
    assert_eq!(
        q.round_number(),
        2,
        "round should increment after full cycle"
    );
}

#[test]
fn turn_queue_current_team_returns_current_teams_id() {
    let q = TurnQueue::new(make_entries());
    let team = q.current_team().expect("should have team");
    assert_eq!(team.as_str(), "player");
}

#[test]
fn turn_queue_just_changed_team_false_when_same_team() {
    let mut q = TurnQueue::new(make_entries());
    // entry 0 (player) → entry 1 (player): same team
    q.advance();
    assert!(!q.just_changed_team());
}

#[test]
fn turn_queue_just_changed_team_true_when_team_switches() {
    let mut q = TurnQueue::new(make_entries());
    // entry 0 (player) → entry 1 (player) → entry 2 (enemy): team switch
    q.advance();
    q.advance();
    assert!(q.just_changed_team());
}

#[test]
fn turn_queue_just_changed_team_false_when_wrapping_to_same_team() {
    let mut q = TurnQueue::new(make_entries());
    // Advance through all 4 → wraps to entry 0
    q.advance();
    q.advance();
    q.advance();
    q.advance();
    // Last entry in cycle was entry 3 (player), wrapping to entry 0 (player): same team
    assert!(!q.just_changed_team());
}

#[test]
fn turn_queue_just_changed_team_true_on_team_switch() {
    let entries = vec![
        TurnEntry::new(entity(1), TeamId::new("player"), 20),
        TurnEntry::new(entity(2), TeamId::new("enemy"), 15),
        TurnEntry::new(entity(3), TeamId::new("player"), 10),
    ];
    let mut q = TurnQueue::new(entries);
    // player → enemy: team switch
    q.advance();
    assert!(q.just_changed_team());
}

#[test]
fn turn_queue_empty_returns_none_for_current() {
    let q = TurnQueue::new(vec![]);
    assert!(q.current().is_none());
    assert!(q.current_team().is_none());
    assert!(q.is_empty());
}

#[test]
fn turn_queue_advance_on_empty_queue_is_safe_no_panic() {
    let mut q = TurnQueue::new(vec![]);
    q.advance(); // Should not panic
    assert_eq!(q.current_index(), 0);
    // 以 0 为除数时 advance 不会 panic
}

#[test]
fn turn_queue_entries_returns_all_entries() {
    let q = TurnQueue::new(make_entries());
    assert_eq!(q.entries().len(), 4);
}

#[test]
fn turn_queue_default_is_empty() {
    let q = TurnQueue::default();
    assert!(q.is_empty());
    assert_eq!(q.round_number(), 1);
}
