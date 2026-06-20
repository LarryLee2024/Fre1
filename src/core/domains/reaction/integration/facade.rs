//! ReactionReadFacade + ReactionWriteFacade — Reaction 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Reaction 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Reaction 域组件的修改操作，使用两种方式：
//! - `&mut World` 方法：立即执行，适合独占 System / 测试
//! - `Commands` 方法：延迟执行，适合常规 System
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::reaction::components::ReactionState;
use crate::core::domains::reaction::failure::ReactionFailure;
use crate::core::domains::reaction::resources::{GlobalReactionQueue, ReactionConfig};

// ─── ReactionReadFacade ───────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Reaction 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct ReactionReadFacade;

impl ReactionReadFacade {
    /// 获取实体的反应槽位状态。
    ///
    /// # Returns
    /// - `Some(&ReactionState)` — 如果实体拥有 `ReactionState` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询反应槽位状态
    pub fn get_reaction_state(world: &World, entity: Entity) -> Option<&ReactionState> {
        world.get::<ReactionState>(entity)
    }

    /// 检查实体当前是否可以使用反应。
    ///
    /// 委托给 `ReactionState::can_react()`。
    ///
    /// # Returns
    /// - `true` — 实体有未使用的反应次数
    /// - `false` — 实体无 `ReactionState` 组件或已用尽反应次数
    ///
    /// # ReadFacade: 安全校验反应可用性
    pub fn can_react(world: &World, entity: Entity) -> bool {
        world
            .get::<ReactionState>(entity)
            .is_some_and(|state| state.can_react())
    }

    /// 获取反应系统全局配置。
    ///
    /// # ReadFacade: 安全查询全局配置
    pub fn get_reaction_config(world: &World) -> &ReactionConfig {
        world.resource::<ReactionConfig>()
    }

    /// 获取全局反应队列。
    ///
    /// # ReadFacade: 安全查询全局队列
    pub fn get_global_queue(world: &World) -> &GlobalReactionQueue {
        world.resource::<GlobalReactionQueue>()
    }

    /// 检查全局反应队列是否已处理完毕。
    ///
    /// # ReadFacade: 安全查询队列完成状态
    pub fn is_queue_finished(world: &World) -> bool {
        world.resource::<GlobalReactionQueue>().queue.is_finished()
    }
}

// ─── ReactionWriteFacade ───────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Reaction 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct ReactionWriteFacade;

impl ReactionWriteFacade {
    /// 消耗实体的一次反应机会（立即执行）。
    ///
    /// 委托给 `ReactionState::consume()`。
    ///
    /// # Errors
    /// - `ReactionFailure::NoReactionsAvailable` — 反应次数已用尽或实体无 `ReactionState` 组件
    ///
    /// # WriteFacade: 安全消耗反应机会
    pub fn consume_reaction(world: &mut World, entity: Entity) -> Result<(), ReactionFailure> {
        let mut state = world
            .get_mut::<ReactionState>(entity)
            .ok_or(ReactionFailure::NoReactionsAvailable { reactor: entity })?;

        if state.consume() {
            Ok(())
        } else {
            Err(ReactionFailure::NoReactionsAvailable { reactor: entity })
        }
    }

    /// 重置实体的反应槽位状态（新回合开始时调用，立即执行）。
    ///
    /// # WriteFacade: 安全重置单实体反应状态
    pub fn reset_reactions(world: &mut World, entity: Entity) {
        if let Some(mut state) = world.get_mut::<ReactionState>(entity) {
            state.reset();
        }
    }

    /// 重置所有实体的反应槽位状态（新回合开始时调用，立即执行）。
    ///
    /// # WriteFacade: 安全批量重置反应状态
    pub fn reset_all_reactions(world: &mut World) {
        let mut query = world.query::<&mut ReactionState>();
        for mut state in query.iter_mut(world) {
            state.reset();
        }
    }

    /// 清空全局反应队列（帧末清理，立即执行）。
    ///
    /// # WriteFacade: 安全清理队列
    pub fn clear_global_queue(world: &mut World) {
        if let Some(mut queue) = world.get_resource_mut::<GlobalReactionQueue>() {
            queue.clear();
        }
    }

    /// 清除所有实体的反应槽位状态（移除组件，通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 移除反应状态
    pub fn remove_all_reaction_states(commands: &mut Commands, entities: &[Entity]) {
        for &entity in entities {
            commands.entity(entity).remove::<ReactionState>();
        }
    }

    /// 初始化实体的反应槽位状态（通过 Commands 延迟执行）。
    ///
    /// 为实体添加默认的 `ReactionState` 组件。
    ///
    /// # WriteFacade: 通过 Commands 初始化反应状态
    pub fn init_reaction_state(commands: &mut Commands, entity: Entity) {
        commands.entity(entity).insert(ReactionState::new());
    }

    /// 设置实体的额外反应次数（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 设置额外反应
    pub fn set_extra_reactions(commands: &mut Commands, entity: Entity, extra: u32) {
        commands.entity(entity).insert(ReactionState {
            used: false,
            extra_reactions: extra,
            extra_used: 0,
        });
    }
}
