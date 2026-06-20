//! SpellQueryParam — Bevy SystemParam，封装所有 Spell 域组件查询。
//!
//! Systems 通过此 param 读取法术数据，完全不知道 `SpellSlotPool` /
//! `Spellbook` / `Concentration` 组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     spell_query: SpellQueryParam,
//!     // ...
//! ) {
//!     if let Some(spellbook) = spell_query.get_spellbook(entity) {
//!         // 读取法术书
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `SpellWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::spell::components::{
    Concentration, SpellLevel, SpellSlotPool, Spellbook,
};

/// 法术查询 SystemParam — 封装所有 Spell 域组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&Spellbook>` + `Query<&SpellSlotPool>`。
#[derive(SystemParam)]
pub struct SpellQueryParam<'w, 's> {
    /// 法术书只读查询
    spellbook_query: Query<'w, 's, &'static Spellbook>,
    /// 法术位池只读查询
    slot_pool_query: Query<'w, 's, &'static SpellSlotPool>,
    /// 专注状态只读查询
    concentration_query: Query<'w, 's, &'static Concentration>,
}

impl<'w, 's> SpellQueryParam<'w, 's> {
    /// 获取实体的法术书。
    ///
    /// # Returns
    /// - `Some(&Spellbook)` — 如果实体拥有 `Spellbook` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_spellbook(&self, entity: Entity) -> Option<&Spellbook> {
        self.spellbook_query.get(entity).ok()
    }

    /// 获取实体的法术位池。
    ///
    /// # Returns
    /// - `Some(&SpellSlotPool)` — 如果实体拥有 `SpellSlotPool` 组件
    /// - `None` — 如果实体不存在或无该组件
    pub fn get_slot_pool(&self, entity: Entity) -> Option<&SpellSlotPool> {
        self.slot_pool_query.get(entity).ok()
    }

    /// 检查实体是否正在进行专注施法。
    pub fn has_concentration(&self, entity: Entity) -> bool {
        self.concentration_query.get(entity).is_ok()
    }

    /// 获取实体的专注状态详情。
    ///
    /// # Returns
    /// - `Some(&Concentration)` — 如果实体正在专注
    /// - `None` — 无专注状态
    pub fn get_concentration(&self, entity: Entity) -> Option<&Concentration> {
        self.concentration_query.get(entity).ok()
    }

    /// 查询实体在指定环阶的剩余法术位数。
    pub fn remaining_slots(&self, entity: Entity, level: SpellLevel) -> u32 {
        self.slot_pool_query
            .get(entity)
            .map_or(0, |pool| pool.remaining(level))
    }
}
