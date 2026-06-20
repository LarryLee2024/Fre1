//! CampRestQueryParam — Bevy SystemParam，封装所有 CampRest 域组件查询。
//!
//! Systems 通过此 param 读取营地/休息数据，完全不知道
//! `RestState` / `HitDicePool` / `CampNPC` 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     camp_rest_query: CampRestQueryParam,
//!     // ...
//! ) {
//!     if let Some(state) = camp_rest_query.get_rest_state(entity) {
//!         // 读取休息状态
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `CampRestWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::camp_rest::components::{
    CampNPC, CampRestMarker, DiceType, HitDicePool, RestPhase, RestState,
};

/// 营地/休息查询 SystemParam — 封装所有 CampRest 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&RestState>` + `Query<&HitDicePool>`。
#[derive(SystemParam)]
pub struct CampRestQueryParam<'w, 's> {
    /// 休息状态只读查询
    rest_state_query: Query<'w, 's, &'static RestState>,
    /// 生命骰池只读查询
    hit_dice_query: Query<'w, 's, &'static HitDicePool>,
    /// 营地 NPC 只读查询
    camp_npc_query: Query<'w, 's, &'static CampNPC>,
    /// 休息标记只读查询
    marker_query: Query<'w, 's, &'static CampRestMarker>,
}

impl<'w, 's> CampRestQueryParam<'w, 's> {
    /// 获取实体的休息状态。
    ///
    /// # Returns
    /// - `Some(&RestState)` — 如果实体拥有 `RestState` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_rest_state(&self, entity: Entity) -> Option<&RestState> {
        self.rest_state_query.get(entity).ok()
    }

    /// 获取实体的生命骰池。
    ///
    /// # Returns
    /// - `Some(&HitDicePool)` — 如果实体拥有 `HitDicePool` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_hit_dice_pool(&self, entity: Entity) -> Option<&HitDicePool> {
        self.hit_dice_query.get(entity).ok()
    }

    /// 获取实体的营地 NPC 数据。
    ///
    /// # Returns
    /// - `Some(&CampNPC)` — 如果实体拥有 `CampNPC` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_camp_npc(&self, entity: Entity) -> Option<&CampNPC> {
        self.camp_npc_query.get(entity).ok()
    }

    /// 检查实体是否拥有 CampRestMarker。
    pub fn has_marker(&self, entity: Entity) -> bool {
        self.marker_query.get(entity).is_ok()
    }

    /// 检查实体是否正在进行休息。
    ///
    /// 返回 `true` 当实体处于 Resting 或 LightActivity 阶段。
    pub fn is_resting(&self, entity: Entity) -> bool {
        self.rest_state_query
            .get(entity)
            .is_ok_and(|state| state.phase.is_resting())
    }

    /// 获取实体当前的休息阶段。
    ///
    /// # Returns
    /// - `Some(RestPhase)` — 如果实体拥有 `RestState` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn current_phase(&self, entity: Entity) -> Option<RestPhase> {
        self.rest_state_query
            .get(entity)
            .ok()
            .map(|state| state.phase)
    }

    /// 获取实体当前可用的生命骰数量。
    pub fn remaining_hit_dice(&self, entity: Entity) -> u32 {
        self.hit_dice_query
            .get(entity)
            .map_or(0, |pool| pool.current)
    }

    /// 获取实体生命骰池完整信息（current, max, dice_type）。
    ///
    /// # Returns
    /// - `Some((current, max, dice_type))` — 如果实体拥有 `HitDicePool` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn hit_dice_info(&self, entity: Entity) -> Option<(u32, u32, DiceType)> {
        self.hit_dice_query
            .get(entity)
            .ok()
            .map(|pool| (pool.current, pool.max, pool.dice_type))
    }
}
