//! Quest Domain — 任务生命周期集成测试
//!
//! 验证 QuestPlugin 注册后的 Observer 链路：
//! - 接受任务 → on_accept_quest_request → QuestLog 新增条目
//! - 推进目标 → on_advance_objective → ObjectiveProgress 更新
//! - 交付任务 → on_turn_in_quest → 标记已完成
//! - 任务失败 → on_quest_failed → 标记失败

use bevy::prelude::*;

use crate::core::domains::quest::components::{
    ObjectiveDef, ObjectiveId, QuestDefId, QuestLog, QuestRewardDef, QuestState,
};
use crate::core::domains::quest::events::{
    QuestAccepted, QuestFailed, QuestProgressUpdated, QuestTurnedIn,
};
use crate::core::domains::quest::plugin::QuestPlugin;

// ─── 辅助函数 ──────────────────────────────────────────────────────

fn spawn_quest_log(world: &mut World) -> Entity {
    world.spawn(QuestLog::new()).id()
}

fn make_objective(id: &str, target: u32) -> ObjectiveDef {
    ObjectiveDef {
        id: ObjectiveId(id.to_string()),
        description_key: format!("obj.{}.desc", id),
        objective_type: crate::core::domains::quest::components::ObjectiveType::Kill {
            enemy_tags: vec!["goblin".into()],
        },
        target_value: target,
        associated_id: None,
    }
}

// ─── 接受任务 ──────────────────────────────────────────────────────

#[test]
fn accept_quest_adds_entry_to_log() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    app.world_mut().trigger(QuestAccepted {
        entity,
        quest_id: QuestDefId::new("qst_kill_goblins"),
        objectives: vec![make_objective("obj_kill_1", 5)],
    });
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let entry = log.get_entry(&QuestDefId::new("qst_kill_goblins"));
    assert!(entry.is_some(), "接受任务后应有对应条目");
    if let Some(e) = entry {
        assert_eq!(
            e.state,
            QuestState::Active,
            "新接受的任务应处于 Active 状态"
        );
    }
}

#[test]
fn accept_quest_twice_keeps_single_entry() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    // 两次接受同一任务
    for _ in 0..2 {
        app.world_mut().trigger(QuestAccepted {
            entity,
            quest_id: QuestDefId::new("qst_double"),
            objectives: vec![],
        });
    }
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let active_count = log
        .entries
        .iter()
        .filter(|e| e.quest_id == QuestDefId::new("qst_double"))
        .count();
    assert_eq!(active_count, 1, "重复接受同一任务应只有一条记录");
}

// ─── 推进目标 ──────────────────────────────────────────────────────

#[test]
fn advance_objective_progresses_correctly() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    // 先接受任务（目标需要 5 击杀）
    app.world_mut().trigger(QuestAccepted {
        entity,
        quest_id: QuestDefId::new("qst_slayer"),
        objectives: vec![make_objective("obj_kill", 5)],
    });
    app.world_mut().flush();

    // 推进目标从 0 → 3
    app.world_mut().trigger(QuestProgressUpdated {
        entity,
        quest_id: QuestDefId::new("qst_slayer"),
        objective_id: "obj_kill".into(),
        old_progress: 0,
        new_progress: 3,
        target: 5,
    });
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let entry = log.get_entry(&QuestDefId::new("qst_slayer")).unwrap();
    let progress = entry
        .objective_progress
        .iter()
        .find(|p| p.objective_id == ObjectiveId("obj_kill".into()))
        .unwrap();
    assert_eq!(progress.current_value, 3, "目标进度应为 3");
    assert!(!progress.is_completed, "进度 3/5 不应已完成");
}

#[test]
fn advance_objective_completes_when_target_reached() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    app.world_mut().trigger(QuestAccepted {
        entity,
        quest_id: QuestDefId::new("qst_collector"),
        objectives: vec![make_objective("obj_collect", 3)],
    });
    app.world_mut().flush();

    // 直接推进到完成：0 → 3
    app.world_mut().trigger(QuestProgressUpdated {
        entity,
        quest_id: QuestDefId::new("qst_collector"),
        objective_id: "obj_collect".into(),
        old_progress: 0,
        new_progress: 3,
        target: 3,
    });
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let entry = log.get_entry(&QuestDefId::new("qst_collector")).unwrap();
    let progress = entry
        .objective_progress
        .iter()
        .find(|p| p.objective_id == ObjectiveId("obj_collect".into()))
        .unwrap();
    assert!(progress.is_completed, "达到目标值后应标记完成");
    assert_eq!(
        progress.current_value, 3,
        "完成时 current_value 应等于 target_value"
    );
}

#[test]
fn completed_quest_ignores_further_progress() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    // 接受、推进到完成、交付
    let qid = QuestDefId::new("qst_done");
    app.world_mut().trigger(QuestAccepted {
        entity,
        quest_id: qid.clone(),
        objectives: vec![make_objective("obj_1", 1)],
    });
    app.world_mut().flush();

    app.world_mut().trigger(QuestProgressUpdated {
        entity,
        quest_id: qid.clone(),
        objective_id: "obj_1".into(),
        old_progress: 0,
        new_progress: 1,
        target: 1,
    });
    app.world_mut().flush();

    // 交付
    app.world_mut().trigger(QuestTurnedIn {
        entity,
        quest_id: qid.clone(),
        rewards: QuestRewardDef {
            xp_reward: 100,
            gold_reward: 50,
            item_rewards: vec![],
            reputation_rewards: vec![],
            unlocks: vec![],
        },
    });
    app.world_mut().flush();

    // 再次推进（已完成的条目应忽略）
    app.world_mut().trigger(QuestProgressUpdated {
        entity,
        quest_id: qid.clone(),
        objective_id: "obj_1".into(),
        old_progress: 1,
        new_progress: 5,
        target: 1,
    });
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let entry = log.get_entry(&qid).unwrap();
    assert_eq!(
        entry.state,
        QuestState::Completed,
        "已完成的任务状态不应被后续进度变更"
    );
}

// ─── 交付任务 ──────────────────────────────────────────────────────

#[test]
fn turn_in_completable_quest_marks_completed() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    let qid = QuestDefId::new("qst_turnin");
    app.world_mut().trigger(QuestAccepted {
        entity,
        quest_id: qid.clone(),
        objectives: vec![make_objective("obj_a", 1)],
    });
    app.world_mut().flush();

    // 完成目标
    app.world_mut().trigger(QuestProgressUpdated {
        entity,
        quest_id: qid.clone(),
        objective_id: "obj_a".into(),
        old_progress: 0,
        new_progress: 1,
        target: 1,
    });
    app.world_mut().flush();

    // 交付
    app.world_mut().trigger(QuestTurnedIn {
        entity,
        quest_id: qid.clone(),
        rewards: QuestRewardDef {
            xp_reward: 100,
            gold_reward: 50,
            item_rewards: vec![],
            reputation_rewards: vec![],
            unlocks: vec![],
        },
    });
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let entry = log.get_entry(&qid).unwrap();
    assert_eq!(entry.state, QuestState::Completed);
    assert_eq!(log.completed_count, 1, "completed_count 应增加");
}

// ─── 任务失败 ──────────────────────────────────────────────────────

#[test]
fn fail_quest_marks_as_failed() {
    let mut app = App::new();
    app.add_plugins(QuestPlugin);

    let entity = spawn_quest_log(app.world_mut());
    app.world_mut().flush();

    let qid = QuestDefId::new("qst_failable");
    app.world_mut().trigger(QuestAccepted {
        entity,
        quest_id: qid.clone(),
        objectives: vec![],
    });
    app.world_mut().flush();

    app.world_mut().trigger(QuestFailed {
        entity,
        quest_id: qid.clone(),
        fail_reason: "目标 NPC 死亡".into(),
    });
    app.world_mut().flush();

    let log = app.world_mut().get::<QuestLog>(entity).unwrap();
    let entry = log.get_entry(&qid).unwrap();
    assert_eq!(entry.state, QuestState::Failed, "任务应标记为 Failed");
    assert_eq!(
        entry.fail_reason,
        Some("目标 NPC 死亡".into()),
        "失败原因应被记录"
    );
}
