//! QuestQueryParam — Bevy SystemParam，封装所有 Quest 域组件查询。
//!
//! Systems 通过此 param 读取任务数据，完全不知道 `QuestLog` / `QuestEntry`
//! 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     quest_query: QuestQueryParam,
//!     // ...
//! ) {
//!     if let Some(log) = quest_query.get_quest_log(entity) {
//!         // 读取任务日志
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `QuestWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::quest::components::{QuestEntry, QuestLog, QuestState};
use crate::shared::ids::QuestId as QuestDefId;

/// 任务查询 SystemParam — 封装所有 Quest 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&QuestLog>`。
#[derive(SystemParam)]
pub struct QuestQueryParam<'w, 's> {
    /// 任务日志只读查询
    quest_log_query: Query<'w, 's, &'static QuestLog>,
}

impl<'w, 's> QuestQueryParam<'w, 's> {
    /// 获取实体的任务日志组件。
    ///
    /// # Returns
    /// - `Some(&QuestLog)` — 如果实体拥有 `QuestLog` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_quest_log(&self, entity: Entity) -> Option<&QuestLog> {
        self.quest_log_query.get(entity).ok()
    }

    /// 获取任务日志中指定任务的条目。
    ///
    /// # Returns
    /// - `Some(&QuestEntry)` — 如果任务存在于日志中
    /// - `None` — 如果实体无 QuestLog 或任务不存在
    pub fn get_entry(&self, entity: Entity, quest_id: &QuestDefId) -> Option<&QuestEntry> {
        self.quest_log_query
            .get(entity)
            .ok()
            .and_then(|log| log.get_entry(quest_id))
    }

    /// 获取任务日志中指定任务的当前状态。
    ///
    /// # Returns
    /// - `Some(QuestState)` — 任务当前状态（任务存在于日志中）
    /// - `None` — 如果实体无 QuestLog 或任务不存在
    pub fn get_entry_state(&self, entity: Entity, quest_id: &QuestDefId) -> Option<QuestState> {
        self.quest_log_query
            .get(entity)
            .ok()
            .and_then(|log| log.get_entry(quest_id))
            .map(|entry| entry.state.clone())
    }

    /// 检查任务日志中是否存在指定任务。
    pub fn has_quest(&self, entity: Entity, quest_id: &QuestDefId) -> bool {
        self.quest_log_query
            .get(entity)
            .ok()
            .is_some_and(|log| log.get_entry(quest_id).is_some())
    }

    /// 检查指定任务是否已完成。
    pub fn is_quest_completed(&self, entity: Entity, quest_id: &QuestDefId) -> bool {
        self.quest_log_query
            .get(entity)
            .ok()
            .and_then(|log| log.get_entry(quest_id))
            .is_some_and(|entry| entry.state == QuestState::Completed)
    }

    /// 获取任务日志中所有已完成的任务数。
    pub fn get_completed_count(&self, entity: Entity) -> u32 {
        self.quest_log_query
            .get(entity)
            .map_or(0, |log| log.completed_count)
    }
}
