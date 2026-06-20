//! SummonQueryParam — Bevy SystemParam，封装所有 Summon 域组件查询。
//!
//! Systems 通过此 param 读取召唤数据，完全不知道 `SummonBond` /
//! `SummonSlotManager` 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     summon_query: SummonQueryParam,
//!     // ...
//! ) {
//!     if let Some(bond) = summon_query.get_summon_bond(entity) {
//!         // 读取召唤绑定关系
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `SummonWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::summon::components::{SummonBond, SummonSlotManager};

/// 召唤查询 SystemParam — 封装所有 Summon 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&SummonBond>` + `Query<&SummonSlotManager>`。
#[derive(SystemParam)]
pub struct SummonQueryParam<'w, 's> {
    /// 召唤绑定只读查询
    summon_bond_query: Query<'w, 's, &'static SummonBond>,
    /// 召唤槽位管理器只读查询
    slot_manager_query: Query<'w, 's, &'static SummonSlotManager>,
}

impl<'w, 's> SummonQueryParam<'w, 's> {
    /// 获取实体的召唤绑定信息。
    ///
    /// # Returns
    /// - `Some(&SummonBond)` — 如果实体拥有 `SummonBond` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_summon_bond(&self, entity: Entity) -> Option<&SummonBond> {
        self.summon_bond_query.get(entity).ok()
    }

    /// 获取实体的召唤槽位管理器。
    ///
    /// # Returns
    /// - `Some(&SummonSlotManager)` — 如果实体拥有 `SummonSlotManager` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_slot_manager(&self, entity: Entity) -> Option<&SummonSlotManager> {
        self.slot_manager_query.get(entity).ok()
    }

    /// 检查实体是否有空闲召唤槽位。
    ///
    /// 如果实体没有 `SummonSlotManager` 组件，默认为无空闲槽位。
    pub fn has_free_slot(&self, entity: Entity) -> bool {
        self.slot_manager_query
            .get(entity)
            .is_ok_and(|mgr| mgr.has_free_slot())
    }

    /// 获取实体当前的召唤物数量。
    ///
    /// 如果实体没有 `SummonSlotManager` 组件，返回 0。
    pub fn active_summon_count(&self, entity: Entity) -> u32 {
        self.slot_manager_query
            .get(entity)
            .map_or(0, |mgr| mgr.active_summons.len() as u32)
    }
}
