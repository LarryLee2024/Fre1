//! ReactionQueryParam — Bevy SystemParam，封装所有 Reaction 域组件查询。
//!
//! Systems 通过此 param 读取反应数据，完全不知道 `ReactionState` 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     reaction_query: ReactionQueryParam,
//!     // ...
//! ) {
//!     if let Some(state) = reaction_query.get_reaction_state(entity) {
//!         // 读取反应状态
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `ReactionWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::reaction::components::ReactionState;

/// 反应查询 SystemParam — 封装所有 Reaction 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&ReactionState>`。
#[derive(SystemParam)]
pub struct ReactionQueryParam<'w, 's> {
    /// 反应槽位状态只读查询
    reaction_state_query: Query<'w, 's, &'static ReactionState>,
}

impl<'w, 's> ReactionQueryParam<'w, 's> {
    /// 获取实体的反应槽位状态。
    ///
    /// # Returns
    /// - `Some(&ReactionState)` — 如果实体拥有 `ReactionState` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_reaction_state(&self, entity: Entity) -> Option<&ReactionState> {
        self.reaction_state_query.get(entity).ok()
    }

    /// 检查实体当前是否可以使用反应。
    ///
    /// 委托给 `ReactionState::can_react()`.
    ///
    /// # Returns
    /// - `true` — 实体有未使用的反应次数
    /// - `false` — 实体无 `ReactionState` 组件或已用尽反应次数
    pub fn can_react(&self, entity: Entity) -> bool {
        self.reaction_state_query
            .get(entity)
            .is_ok_and(|state| state.can_react())
    }

    /// 检查实体是否拥有反应槽位组件。
    ///
    /// # Returns
    /// - `true` — 实体拥有 `ReactionState` 组件
    /// - `false` — 实体无该组件
    pub fn has_reaction_state(&self, entity: Entity) -> bool {
        self.reaction_state_query.get(entity).is_ok()
    }

    /// 检查实体是否还有额外反应次数。
    ///
    /// # Returns
    /// - `true` — 实体已用完基本反应且有剩余额外反应
    /// - `false` — 实体无 `ReactionState` 组件或已用尽所有反应
    pub fn has_extra_reactions(&self, entity: Entity) -> bool {
        self.reaction_state_query
            .get(entity)
            .is_ok_and(|state| state.used && state.extra_used < state.extra_reactions)
    }
}
