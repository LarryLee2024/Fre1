//! NarrativeQueryParam — Bevy SystemParam，封装所有 Narrative 域组件查询。
//!
//! Systems 通过此 param 读取叙事数据，完全不知道 `DialogueState` /
//! `StoryFlags` / `CutsceneState` / `DialogueHistory` / `DialogueTreeRegistry`
//! 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     narrative_query: NarrativeQueryParam,
//!     // ...
//! ) {
//!     if let Some(dialogue) = narrative_query.get_dialogue_state(entity) {
//!         // 读取对话状态
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `NarrativeWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::narrative::components::{
    CutsceneState, DialogueHistory, DialogueState, DialogueTreeRegistry, StoryFlags,
};

/// 叙事查询 SystemParam — 封装所有 Narrative 域组件和资源查询。
///
/// System 签名中使用此类型替代裸 `Query<&DialogueState>` + `Res<DialogueHistory>`。
#[derive(SystemParam)]
pub struct NarrativeQueryParam<'w, 's> {
    /// 对话状态只读查询
    dialogue_state_query: Query<'w, 's, &'static DialogueState>,
    /// 故事标记只读查询
    story_flags_query: Query<'w, 's, &'static StoryFlags>,
    /// 演出状态只读查询
    cutscene_state_query: Query<'w, 's, &'static CutsceneState>,
    /// 对话历史资源
    dialogue_history: Res<'w, DialogueHistory>,
    /// 对话树注册表资源
    dialogue_tree_registry: Res<'w, DialogueTreeRegistry>,
}

impl<'w, 's> NarrativeQueryParam<'w, 's> {
    /// 获取实体的当前对话状态。
    ///
    /// # Returns
    /// - `Some(&DialogueState)` — 如果实体拥有 `DialogueState` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_dialogue_state(&self, entity: Entity) -> Option<&DialogueState> {
        self.dialogue_state_query.get(entity).ok()
    }

    /// 获取实体的故事标记集合。
    ///
    /// # Returns
    /// - `Some(&StoryFlags)` — 如果实体拥有 `StoryFlags` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_story_flags(&self, entity: Entity) -> Option<&StoryFlags> {
        self.story_flags_query.get(entity).ok()
    }

    /// 获取实体的演出状态。
    ///
    /// # Returns
    /// - `Some(&CutsceneState)` — 如果实体拥有 `CutsceneState` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_cutscene_state(&self, entity: Entity) -> Option<&CutsceneState> {
        self.cutscene_state_query.get(entity).ok()
    }

    /// 检查实体是否有活跃的对话会话。
    pub fn has_active_dialogue(&self, entity: Entity) -> bool {
        self.dialogue_state_query.get(entity).is_ok()
    }

    /// 检查实体的故事标记是否等于指定值。
    pub fn check_story_flag(&self, entity: Entity, flag_id: &str, expected: &str) -> bool {
        self.story_flags_query
            .get(entity)
            .map_or(false, |flags| flags.check(flag_id, expected))
    }

    /// 获取全局对话历史的引用。
    pub fn dialogue_history(&self) -> &DialogueHistory {
        &self.dialogue_history
    }

    /// 获取全局对话树注册表的引用。
    pub fn dialogue_tree_registry(&self) -> &DialogueTreeRegistry {
        &self.dialogue_tree_registry
    }

    /// 检查对话树是否可以被跳过（已读）。
    pub fn can_skip_dialogue(&self, tree_id: &str) -> bool {
        self.dialogue_history.can_skip(tree_id)
    }
}
