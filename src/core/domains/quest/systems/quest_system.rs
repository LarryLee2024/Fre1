//! 任务管理 Systems
//!
//! 包括任务接取、进度更新、奖励发放等 System。
//! 详见 docs/02-domain/domains/quest_domain.md §5

use bevy::prelude::*;

use super::super::components::{QuestEntry, QuestLog, QuestState};
use super::super::events::{
    ObjectiveCompleted, QuestAccepted, QuestFailed, QuestProgressUpdated, QuestTurnedIn,
};
use super::super::rules::can_turn_in;

/// 监听任务接受请求的 Observer。
pub fn on_accept_quest_request(_trigger: On<QuestAccepted>, mut query: Query<&mut QuestLog>) {
    let event = _trigger.event();
    if let Ok(mut quest_log) = query.get_mut(event.entity) {
        let entry = QuestEntry::new(event.quest_id.clone(), event.objectives.clone());
        if let Some(existing) = quest_log.get_entry_mut(&event.quest_id) {
            existing.state = QuestState::Active;
        } else {
            let mut entry = QuestEntry::new(event.quest_id.clone(), event.objectives.clone());
            entry.state = QuestState::Active;
            quest_log.entries.push(entry);
        }
    }
}

/// 监听进度更新请求的 Observer。
pub fn on_advance_objective(
    _trigger: On<QuestProgressUpdated>,
    mut commands: Commands,
    mut query: Query<&mut QuestLog>,
) {
    let event = _trigger.event();
    if let Ok(mut quest_log) = query.get_mut(event.entity) {
        if let Some(entry) = quest_log.get_entry_mut(&event.quest_id) {
            if entry.state != QuestState::Active {
                return;
            }
            if let Some(progress) = entry
                .objective_progress
                .iter_mut()
                .find(|p| p.objective_id.0 == event.objective_id)
            {
                let just_completed =
                    progress.advance(event.new_progress.saturating_sub(event.old_progress));

                if just_completed {
                    commands.trigger(ObjectiveCompleted {
                        entity: event.entity,
                        quest_id: event.quest_id.clone(),
                        objective_id: event.objective_id.clone(),
                        objective_type: "objective".to_string(),
                    });
                }

                if entry.all_objectives_completed() {
                    // 标记任务为可交付（仍处于 Active，但可交付）
                }
            }
        }
    }
}

/// 交付任务 — 发放奖励并标记完成。
pub fn on_turn_in_quest(_trigger: On<QuestTurnedIn>, mut query: Query<&mut QuestLog>) {
    let event = _trigger.event();
    if let Ok(mut quest_log) = query.get_mut(event.entity) {
        if let Some(entry) = quest_log.get_entry_mut(&event.quest_id) {
            if !can_turn_in(entry) {
                return;
            }
            entry.state = QuestState::Completed;
            quest_log.completed_count += 1;
        }
    }
}

/// 处理任务失败。
pub fn on_quest_failed(_trigger: On<QuestFailed>, mut query: Query<&mut QuestLog>) {
    let event = _trigger.event();
    if let Ok(mut quest_log) = query.get_mut(event.entity) {
        if let Some(entry) = quest_log.get_entry_mut(&event.quest_id) {
            entry.state = QuestState::Failed;
            entry.fail_reason = Some(event.fail_reason.clone());
        }
    }
}
