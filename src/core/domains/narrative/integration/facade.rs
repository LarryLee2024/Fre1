//! NarrativeReadFacade + NarrativeWriteFacade — Narrative 域组件读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Narrative 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Narrative 域组件的修改操作，使用两种方式：
//! - `&mut World` 方法（`_immediate` 后缀）：立即执行，适合独占 System / 测试
//! - `Commands` 方法（无后缀）：延迟执行，适合常规 System
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::narrative::components::{
    ChoiceOption, CutsceneState, DialogueHistory, DialogueState, DialogueTreeRegistry, StoryFlags,
};

// ─── NarrativeReadFacade ────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Narrative 域 ECS 组件的只读访问。
/// 所有方法通过 `&World` 查询组件，不包含业务逻辑。
pub struct NarrativeReadFacade;

impl NarrativeReadFacade {
    /// 获取实体的当前对话状态。
    ///
    /// # Returns
    /// - `Some(&DialogueState)` — 如果实体拥有 `DialogueState` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询对话状态
    pub fn get_dialogue_state(world: &World, entity: Entity) -> Option<&DialogueState> {
        world.get::<DialogueState>(entity)
    }

    /// 获取实体的故事标记集合。
    ///
    /// # Returns
    /// - `Some(&StoryFlags)` — 如果实体拥有 `StoryFlags` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询故事标记
    pub fn get_story_flags(world: &World, entity: Entity) -> Option<&StoryFlags> {
        world.get::<StoryFlags>(entity)
    }

    /// 获取实体的演出状态。
    ///
    /// # Returns
    /// - `Some(&CutsceneState)` — 如果实体拥有 `CutsceneState` 组件
    /// - `None` — 如果实体不存在或无该组件
    ///
    /// # ReadFacade: 安全查询演出状态
    pub fn get_cutscene_state(world: &World, entity: Entity) -> Option<&CutsceneState> {
        world.get::<CutsceneState>(entity)
    }

    /// 检查实体是否有活跃的对话会话。
    ///
    /// # Returns
    /// - `true` — 实体拥有 `DialogueState` 组件
    /// - `false` — 无活跃对话
    ///
    /// # ReadFacade: 检查对话活跃状态
    pub fn has_active_dialogue(world: &World, entity: Entity) -> bool {
        world.get::<DialogueState>(entity).is_some()
    }

    /// 检查实体的故事标记是否等于指定值。
    ///
    /// # ReadFacade: 安全比较故事标记值
    pub fn check_story_flag(world: &World, entity: Entity, flag_id: &str, expected: &str) -> bool {
        world
            .get::<StoryFlags>(entity)
            .map_or(false, |flags| flags.check(flag_id, expected))
    }

    /// 获取全局对话历史。
    ///
    /// # ReadFacade: 安全查询对话历史
    pub fn get_dialogue_history(world: &World) -> &DialogueHistory {
        world.resource::<DialogueHistory>()
    }

    /// 获取全局对话树注册表。
    ///
    /// # ReadFacade: 安全查询对话树注册表
    pub fn get_dialogue_tree_registry(world: &World) -> &DialogueTreeRegistry {
        world.resource::<DialogueTreeRegistry>()
    }

    /// 检查对话树是否可以被跳过（已读）。
    ///
    /// # ReadFacade: 安全检查跳过条件
    pub fn can_skip_dialogue(world: &World, tree_id: &str) -> bool {
        world.resource::<DialogueHistory>().can_skip(tree_id)
    }
}

// ─── NarrativeWriteFacade ───────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Narrative 域 ECS 组件的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct NarrativeWriteFacade;

impl NarrativeWriteFacade {
    // ── &mut World (Immediate) 方法 ──────────────────────────────────

    /// 启动对话（立即执行）。
    ///
    /// 在实体上插入 `DialogueState` 组件。
    /// 如果实体已有该组件，旧的会被覆盖。
    ///
    /// # WriteFacade: 立即启动对话
    pub fn start_dialogue_immediate(
        world: &mut World,
        entity: Entity,
        tree_id: String,
        entry_node_id: String,
        choices: Vec<ChoiceOption>,
        time: f64,
    ) {
        if let Ok(mut entity_cmd) = world.get_entity_mut(entity) {
            entity_cmd.insert(DialogueState::new(tree_id, entry_node_id, choices, time));
        }
    }

    /// 推进对话到下一节点（立即执行）。
    ///
    /// # WriteFacade: 立即推进对话
    pub fn advance_dialogue_immediate(
        world: &mut World,
        entity: Entity,
        node_id: &str,
        choices: Vec<ChoiceOption>,
    ) {
        if let Some(mut dialogue_state) = world.get_mut::<DialogueState>(entity) {
            dialogue_state.advance(node_id, choices);
        }
    }

    /// 结束对话（立即执行）。
    ///
    /// 将 `DialogueState.phase` 设为 `DialoguePhase::End`。
    /// 不自动移除组件——调用方可选择调用 `remove_dialogue_state_immediate`。
    ///
    /// # WriteFacade: 立即结束对话
    pub fn end_dialogue_immediate(world: &mut World, entity: Entity) {
        if let Some(mut dialogue_state) = world.get_mut::<DialogueState>(entity) {
            dialogue_state.end();
        }
    }

    /// 设置故事标记（立即执行）。
    ///
    /// 返回 `false` 表示标记已存在且值不同，拒绝覆盖（不变量 3.3）。
    ///
    /// # WriteFacade: 立即设置故事标记
    pub fn set_story_flag_immediate(
        world: &mut World,
        entity: Entity,
        flag_id: &str,
        value: &str,
    ) -> bool {
        world
            .get_mut::<StoryFlags>(entity)
            .map_or(false, |mut flags| flags.set_flag(flag_id, value))
    }

    /// 启动演出（立即执行）。
    ///
    /// 在实体上插入 `CutsceneState` 组件。
    /// 如果实体已有该组件，旧的会被覆盖。
    ///
    /// # WriteFacade: 立即启动演出
    pub fn start_cutscene_immediate(
        world: &mut World,
        entity: Entity,
        cutscene_id: String,
        duration: f32,
        participants: Vec<Entity>,
    ) {
        if let Ok(mut entity_cmd) = world.get_entity_mut(entity) {
            entity_cmd.insert(CutsceneState::new(cutscene_id, duration, participants));
        }
    }

    /// 暂停演出（立即执行）。
    ///
    /// # WriteFacade: 立即暂停演出
    pub fn pause_cutscene_immediate(world: &mut World, entity: Entity) {
        if let Some(mut cutscene) = world.get_mut::<CutsceneState>(entity) {
            cutscene.pause();
        }
    }

    /// 恢复演出（立即执行）。
    ///
    /// # WriteFacade: 立即恢复演出
    pub fn resume_cutscene_immediate(world: &mut World, entity: Entity) {
        if let Some(mut cutscene) = world.get_mut::<CutsceneState>(entity) {
            cutscene.resume();
        }
    }

    /// 移除对话状态组件（立即执行）。
    ///
    /// # WriteFacade: 立即移除对话状态
    pub fn remove_dialogue_state_immediate(world: &mut World, entity: Entity) {
        if let Ok(mut entity_cmd) = world.get_entity_mut(entity) {
            entity_cmd.remove::<DialogueState>();
        }
    }

    /// 移除演出状态组件（立即执行）。
    ///
    /// # WriteFacade: 立即移除演出状态
    pub fn remove_cutscene_state_immediate(world: &mut World, entity: Entity) {
        if let Ok(mut entity_cmd) = world.get_entity_mut(entity) {
            entity_cmd.remove::<CutsceneState>();
        }
    }

    /// 记录对话节点访问（立即执行）。
    ///
    /// # WriteFacade: 立即记录节点访问
    pub fn record_node_visit(world: &mut World, tree_id: &str, node_id: &str) {
        world
            .resource_mut::<DialogueHistory>()
            .visit_node(tree_id.to_string(), node_id.to_string());
    }

    /// 记录玩家选择（立即执行）。
    ///
    /// # WriteFacade: 立即记录选择
    pub fn record_choice(world: &mut World, dialogue_id: &str, choice_id: &str) {
        world
            .resource_mut::<DialogueHistory>()
            .record_choice(dialogue_id.to_string(), choice_id.to_string());
    }

    // ── Commands (Deferred) 方法 ─────────────────────────────────────

    /// 启动对话（通过 Commands 延迟执行）。
    ///
    /// 在实体上插入 `DialogueState` 组件。
    /// 如果实体已有该组件，旧的会被覆盖。
    ///
    /// # WriteFacade: 通过 Commands 启动对话
    pub fn start_dialogue(
        commands: &mut Commands,
        entity: Entity,
        tree_id: String,
        entry_node_id: String,
        choices: Vec<ChoiceOption>,
        time: f64,
    ) {
        commands
            .entity(entity)
            .insert(DialogueState::new(tree_id, entry_node_id, choices, time));
    }

    /// 设置故事标记（通过 Commands 延迟执行）。
    ///
    /// 通过插入 `StoryFlags` 组件实现。注意：如果实体已有 `StoryFlags`，
    /// 此操作会覆盖整个组件。如需追加标记请使用 `_immediate` 变体。
    ///
    /// # WriteFacade: 通过 Commands 设置故事标记
    pub fn set_story_flag(commands: &mut Commands, entity: Entity, flags: StoryFlags) {
        commands.entity(entity).insert(flags);
    }

    /// 启动演出（通过 Commands 延迟执行）。
    ///
    /// 在实体上插入 `CutsceneState` 组件。
    /// 如果实体已有该组件，旧的会被覆盖。
    ///
    /// # WriteFacade: 通过 Commands 启动演出
    pub fn start_cutscene(
        commands: &mut Commands,
        entity: Entity,
        cutscene_id: String,
        duration: f32,
        participants: Vec<Entity>,
    ) {
        commands
            .entity(entity)
            .insert(CutsceneState::new(cutscene_id, duration, participants));
    }

    /// 移除对话状态组件（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 移除对话状态
    pub fn remove_dialogue_state(commands: &mut Commands, entity: Entity) {
        commands.entity(entity).remove::<DialogueState>();
    }

    /// 移除演出状态组件（通过 Commands 延迟执行）。
    ///
    /// # WriteFacade: 通过 Commands 移除演出状态
    pub fn remove_cutscene_state(commands: &mut Commands, entity: Entity) {
        commands.entity(entity).remove::<CutsceneState>();
    }
}
