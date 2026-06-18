//! Quest Domain — 不变量测试
//!
//! 验证 docs/02-domain/domains/quest_domain.md §3 定义的不变量。

use crate::core::domains::quest::components::{
    ObjectiveDef, ObjectiveId, ObjectiveType, QuestDefId, QuestEntry, QuestLog, QuestState,
};
use crate::core::domains::quest::rules::{
    are_all_objectives_completed, can_abandon_quest, can_transition,
    check_exclusivity, check_progress_monotonic, is_reward_already_granted,
};

/// 不变量 3.2：目标进度不可倒退。
#[test]
fn progress_never_decreases() {
    assert!(!check_progress_monotonic(3, 5), "decreasing progress must be invalid");
}

/// 不变量 3.3：奖励不可重复发放。
#[test]
fn reward_granted_only_once() {
    let mut entry = make_entry();
    entry.state = QuestState::Completed;
    assert!(is_reward_already_granted(&entry), "completed → reward marked as granted");
}

#[test]
fn reward_not_granted_for_incomplete() {
    let entry = make_entry();
    assert!(!is_reward_already_granted(&entry), "non-completed → reward not granted");
}

/// 不变量 3.5：关键任务不可放弃。
#[test]
fn critical_quest_protected() {
    assert!(!can_abandon_quest(true), "critical quest cannot be abandoned");
}

/// 不变量 3.1：前置链完整性 — 不可跳过 Available 直接 Active。
#[test]
fn cannot_skip_available_state() {
    assert!(!can_transition(&QuestState::Unavailable, &QuestState::Active));
    assert!(!can_transition(&QuestState::Unavailable, &QuestState::Completed));
}

/// 不变量 3.4：任务互斥性 — 互斥任务不可同时 Active。
#[test]
fn exclusive_quests_cannot_overlap() {
    let mut quest_log = QuestLog::new();
    let mut entry_a = make_entry();
    entry_a.state = QuestState::Active;
    quest_log.entries.push(entry_a);

    let exclusive_with = vec![QuestDefId::new("qst_000001")];
    let result = check_exclusivity(&QuestDefId::new("qst_000002"), &exclusive_with, &quest_log);
    assert!(result.is_err(), "exclusive quest should conflict");
}

/// 衍生不变量：Completed 是终态。
#[test]
fn completed_is_terminal() {
    assert!(!can_transition(&QuestState::Completed, &QuestState::Active), "completed → active blocked");
    assert!(!can_transition(&QuestState::Completed, &QuestState::Failed), "completed → failed blocked");
}

// ─── Helpers ──────────────────────────────────────────────

fn make_entry() -> QuestEntry {
    let objectives = vec![
        ObjectiveDef {
            id: ObjectiveId("obj_inv_1".into()),
            description_key: "invariant.test.1".into(),
            objective_type: ObjectiveType::Custom,
            target_value: 3,
            associated_id: None,
        },
    ];
    QuestEntry::new(QuestDefId::new("qst_invariant"), objectives)
}
