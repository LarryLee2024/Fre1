//! Quest Domain — 单元测试
//!
//! 验证规则纯函数。

use crate::core::domains::quest::components::{
    ObjectiveDef, ObjectiveId, ObjectiveType, QuestDefId, QuestEntry, QuestLog, QuestState,
};
use crate::core::domains::quest::rules::{
    are_all_objectives_completed, can_abandon_quest, can_transition, can_turn_in,
    check_exclusivity, check_progress_monotonic, is_reward_already_granted,
};

// ============================================================================
// check_progress_monotonic
// ============================================================================

#[test]
fn progress_increases_is_valid() {
    assert!(check_progress_monotonic(5, 3));
}

#[test]
fn progress_stays_same_is_valid() {
    assert!(check_progress_monotonic(5, 5));
}

#[test]
fn progress_decreases_is_invalid() {
    assert!(!check_progress_monotonic(3, 5));
}

// ============================================================================
// can_transition
// ============================================================================

#[test]
fn valid_state_transitions() {
    assert!(can_transition(
        &QuestState::Unavailable,
        &QuestState::Available
    ));
    assert!(can_transition(&QuestState::Available, &QuestState::Active));
    assert!(can_transition(&QuestState::Active, &QuestState::Completed));
    assert!(can_transition(&QuestState::Active, &QuestState::Failed));
    assert!(can_transition(&QuestState::Failed, &QuestState::Available));
}

#[test]
fn invalid_state_transitions() {
    assert!(!can_transition(
        &QuestState::Unavailable,
        &QuestState::Active
    )); // skip Available
    assert!(!can_transition(&QuestState::Completed, &QuestState::Active)); // completed is terminal
    assert!(!can_transition(&QuestState::Completed, &QuestState::Failed));
}

// ============================================================================
// can_abandon_quest
// ============================================================================

#[test]
fn non_critical_can_be_abandoned() {
    assert!(can_abandon_quest(false));
}

#[test]
fn critical_cannot_be_abandoned() {
    assert!(!can_abandon_quest(true));
}

// ============================================================================
// are_all_objectives_completed
// ============================================================================

#[test]
fn all_objectives_completed_returns_true() {
    let mut entry = make_test_entry();
    for progress in &mut entry.objective_progress {
        progress.current_value = progress.target_value;
        progress.is_completed = true;
    }
    assert!(are_all_objectives_completed(&entry));
}

#[test]
fn not_all_objectives_completed_returns_false() {
    let entry = make_test_entry(); // all at 0
    assert!(!are_all_objectives_completed(&entry));
}

// ============================================================================
// can_turn_in
// ============================================================================

#[test]
fn active_with_all_objectives_can_turn_in() {
    let mut entry = make_test_entry();
    entry.state = QuestState::Active;
    for progress in &mut entry.objective_progress {
        progress.current_value = progress.target_value;
        progress.is_completed = true;
    }
    assert!(can_turn_in(&entry));
}

#[test]
fn active_with_incomplete_cannot_turn_in() {
    let mut entry = make_test_entry();
    entry.state = QuestState::Active;
    assert!(!can_turn_in(&entry));
}

#[test]
fn non_active_cannot_turn_in() {
    let mut entry = make_test_entry();
    entry.state = QuestState::Available;
    for progress in &mut entry.objective_progress {
        progress.current_value = progress.target_value;
        progress.is_completed = true;
    }
    assert!(!can_turn_in(&entry));
}

// ============================================================================
// is_reward_already_granted
// ============================================================================

#[test]
fn completed_quest_reward_granted() {
    let mut entry = make_test_entry();
    entry.state = QuestState::Completed;
    assert!(is_reward_already_granted(&entry));
}

#[test]
fn active_quest_reward_not_granted() {
    let mut entry = make_test_entry();
    entry.state = QuestState::Active;
    assert!(!is_reward_already_granted(&entry));
}

// ============================================================================
// check_exclusivity
// ============================================================================

#[test]
fn no_exclusive_conflict() {
    let mut quest_log = QuestLog::new();
    let exclusive_with = vec![QuestDefId::new("qst_000002")];
    let result = check_exclusivity(&QuestDefId::new("qst_000001"), &exclusive_with, &quest_log);
    assert!(result.is_ok());
}

#[test]
fn exclusive_quest_active_returns_error() {
    let mut quest_log = QuestLog::new();
    let mut exclusive_entry = make_test_entry();
    exclusive_entry.state = QuestState::Active;
    quest_log.entries.push(exclusive_entry);

    let exclusive_with = vec![QuestDefId::new("qst_000002")];
    let result = check_exclusivity(&QuestDefId::new("qst_000001"), &exclusive_with, &quest_log);
    assert!(result.is_err());
}

// ─── Helpers ──────────────────────────────────────────────────────

fn make_test_entry() -> QuestEntry {
    let objectives = vec![
        ObjectiveDef {
            id: ObjectiveId("obj_001".into()),
            description_key: "kill_test".into(),
            objective_type: ObjectiveType::Custom,
            target_value: 5,
            associated_id: None,
        },
        ObjectiveDef {
            id: ObjectiveId("obj_002".into()),
            description_key: "collect_test".into(),
            objective_type: ObjectiveType::Custom,
            target_value: 3,
            associated_id: None,
        },
    ];
    QuestEntry::new(QuestDefId::new("qst_000001"), objectives)
}
