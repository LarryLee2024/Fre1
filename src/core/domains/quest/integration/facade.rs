//! QuestReadFacade + QuestWriteFacade — Quest 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Quest 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Quest 域组件的修改操作，使用 `&mut World` 立即执行。
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::quest::components::{
    ObjectiveDef, ObjectiveId, ObjectiveProgress, QuestEntry, QuestLog, QuestState,
};
use crate::shared::ids::QuestId as QuestDefId;

// ─── QuestReadFacade ─────────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Quest 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct QuestReadFacade;

impl QuestReadFacade {
    /// 获取实体的任务日志组件。
    ///
    /// # Returns
    /// - `Some(&QuestLog)` — 如果实体拥有 `QuestLog` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询任务日志
    pub fn get_quest_log(world: &World, entity: Entity) -> Option<&QuestLog> {
        world.get::<QuestLog>(entity)
    }

    /// 获取任务日志中指定任务的条目。
    ///
    /// # Returns
    /// - `Some(&QuestEntry)` — 如果任务存在于日志中
    /// - `None` — 如果实体无 QuestLog 或任务不存在
    ///
    /// # ReadFacade: 安全查询任务条目
    pub fn get_entry<'w>(
        world: &'w World,
        entity: Entity,
        quest_id: &QuestDefId,
    ) -> Option<&'w QuestEntry> {
        world
            .get::<QuestLog>(entity)
            .and_then(|log| log.get_entry(quest_id))
    }

    /// 获取任务日志中指定任务的当前状态。
    ///
    /// # Returns
    /// - `Some(QuestState)` — 任务当前状态（任务存在于日志中）
    /// - `None` — 如果实体无 QuestLog 或任务不存在
    ///
    /// # ReadFacade: 安全查询任务状态
    pub fn get_entry_state(
        world: &World,
        entity: Entity,
        quest_id: &QuestDefId,
    ) -> Option<QuestState> {
        world
            .get::<QuestLog>(entity)
            .and_then(|log| log.get_entry(quest_id))
            .map(|entry| entry.state.clone())
    }

    /// 检查任务日志中是否存在指定任务。
    ///
    /// # ReadFacade: 检查任务是否存在
    pub fn has_quest(world: &World, entity: Entity, quest_id: &QuestDefId) -> bool {
        world
            .get::<QuestLog>(entity)
            .is_some_and(|log| log.get_entry(quest_id).is_some())
    }

    /// 检查指定任务是否已完成。
    ///
    /// # ReadFacade: 检查任务完成状态
    pub fn is_quest_completed(world: &World, entity: Entity, quest_id: &QuestDefId) -> bool {
        world
            .get::<QuestLog>(entity)
            .and_then(|log| log.get_entry(quest_id))
            .is_some_and(|entry| entry.state == QuestState::Completed)
    }

    /// 获取任务日志中所有已完成的任务数。
    ///
    /// # ReadFacade: 获取完成计数
    pub fn get_completed_count(world: &World, entity: Entity) -> u32 {
        world
            .get::<QuestLog>(entity)
            .map_or(0, |log| log.completed_count)
    }
}

// ─── QuestWriteFacade ─────────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Quest 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct QuestWriteFacade;

impl QuestWriteFacade {
    /// 向任务日志中添加新任务条目。
    ///
    /// 如果实体无 QuestLog 组件或任务已存在于日志中，则为 no-op。
    ///
    /// # WriteFacade: 安全添加任务条目
    pub fn add_entry(
        world: &mut World,
        entity: Entity,
        quest_id: QuestDefId,
        objectives: Vec<ObjectiveDef>,
    ) {
        if let Some(mut log) = world.get_mut::<QuestLog>(entity)
            && log.get_entry(&quest_id).is_none()
        {
            log.entries.push(QuestEntry::new(quest_id, objectives));
        }
    }

    /// 设置任务日志中指定条目的状态。
    ///
    /// 如果实体无 QuestLog 或任务不存在，则为 no-op。
    ///
    /// # WriteFacade: 安全设置任务状态
    pub fn set_entry_state(
        world: &mut World,
        entity: Entity,
        quest_id: &QuestDefId,
        state: QuestState,
    ) {
        if let Some(mut log) = world.get_mut::<QuestLog>(entity)
            && let Some(entry) = log.get_entry_mut(quest_id)
        {
            entry.state = state;
        }
    }

    /// 推进指定任务中某目标的进度。
    ///
    /// # Returns
    /// - `true` — 目标被找到且进度已推进
    /// - `false` — 实体无 QuestLog、任务不存在、或目标不存在
    ///
    /// # WriteFacade: 安全推进目标进度
    pub fn advance_objective(
        world: &mut World,
        entity: Entity,
        quest_id: &QuestDefId,
        objective_id: &ObjectiveId,
        amount: u32,
    ) -> bool {
        let Some(mut log) = world.get_mut::<QuestLog>(entity) else {
            return false;
        };
        let Some(entry) = log.get_entry_mut(quest_id) else {
            return false;
        };
        let Some(progress) = entry
            .objective_progress
            .iter_mut()
            .find(|p| p.objective_id == *objective_id)
        else {
            return false;
        };
        progress.advance(amount);
        true
    }

    /// 设置任务日志中指定条目的失败原因。
    ///
    /// 如果实体无 QuestLog 或任务不存在，则为 no-op。
    ///
    /// # WriteFacade: 安全设置失败原因
    pub fn set_fail_reason(
        world: &mut World,
        entity: Entity,
        quest_id: &QuestDefId,
        reason: String,
    ) {
        if let Some(mut log) = world.get_mut::<QuestLog>(entity)
            && let Some(entry) = log.get_entry_mut(quest_id)
        {
            entry.fail_reason = Some(reason);
        }
    }

    /// 从任务日志中移除指定任务条目。
    ///
    /// 如果实体无 QuestLog 或任务不存在，则为 no-op。
    ///
    /// # WriteFacade: 安全移除任务条目
    pub fn remove_entry(world: &mut World, entity: Entity, quest_id: &QuestDefId) {
        if let Some(mut log) = world.get_mut::<QuestLog>(entity) {
            log.entries.retain(|e| e.quest_id != *quest_id);
        }
    }

    /// 递增任务日志的已完成任务计数。
    ///
    /// 如果实体无 QuestLog 组件，则为 no-op。
    ///
    /// # WriteFacade: 安全递增完成计数
    pub fn increment_completed_count(world: &mut World, entity: Entity) {
        if let Some(mut log) = world.get_mut::<QuestLog>(entity) {
            log.completed_count = log.completed_count.saturating_add(1);
        }
    }
}
