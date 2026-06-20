//! FactionQueryParam — Bevy SystemParam，封装所有 Faction 组件查询。
//!
//! Systems 通过此 param 获取 Faction 域数据，完全不知道
//! FactionMembership / Reputation / KeyCharacter 组件内部细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     mut faction_query: FactionQueryParam,
//!     // ...
//! ) {
//!     let membership = faction_query.faction_memberships.get(entity);
//!     if let Ok(m) = membership {
//!         println!("Factions: {:?}", m.factions);
//!     }
//! }
//! ```

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::faction::components::{
    FactionId, FactionMembership, FactionRelationTable, FactionRelationType, KeyCharacter,
    Reputation, ReputationLevel,
};

/// Faction 查询参数 — 封装所有 Faction 组件查询。
///
/// System 签名中使用此类型替代裸 `Query<&FactionMembership>` + `Res<FactionRelationTable>`。
/// 函数体内所有 Faction 数据访问都通过此 param 的字段完成。
#[derive(SystemParam)]
pub struct FactionQueryParam<'w, 's> {
    /// FactionMembership 组件查询。
    pub faction_memberships: Query<'w, 's, &'static FactionMembership>,
    /// Reputation 组件查询。
    pub reputations: Query<'w, 's, &'static Reputation>,
    /// KeyCharacter 标记查询。
    pub key_characters: Query<'w, 's, &'static KeyCharacter>,
    /// 阵营关系表资源。
    pub faction_relation_table: Res<'w, FactionRelationTable>,
}

impl<'w, 's> FactionQueryParam<'w, 's> {
    /// 获取实体的 FactionMembership 组件引用。
    pub fn membership(&self, entity: Entity) -> Option<&FactionMembership> {
        self.faction_memberships.get(entity).ok()
    }

    /// 获取实体的 Reputation 组件引用。
    pub fn reputation(&self, entity: Entity) -> Option<&Reputation> {
        self.reputations.get(entity).ok()
    }

    /// 检查实体是否拥有 KeyCharacter 标记。
    pub fn is_key_character(&self, entity: Entity) -> bool {
        self.key_characters.get(entity).is_ok()
    }

    /// 检查实体是否属于指定阵营。
    ///
    /// 返回 `None` 如果实体没有 FactionMembership 组件。
    pub fn has_faction(&self, entity: Entity, faction: &FactionId) -> Option<bool> {
        self.faction_memberships
            .get(entity)
            .ok()
            .map(|m| m.is_member(faction))
    }

    /// 获取实体在指定阵营的声望值。
    ///
    /// 返回 `None` 如果实体没有 Reputation 组件。
    pub fn reputation_value(&self, entity: Entity, faction: &FactionId) -> Option<i32> {
        self.reputations.get(entity).ok().map(|r| r.get(faction))
    }

    /// 获取实体在指定阵营的声望等级。
    ///
    /// 返回 `None` 如果实体没有 Reputation 组件。
    pub fn reputation_level(&self, entity: Entity, faction: &FactionId) -> Option<ReputationLevel> {
        self.reputations.get(entity).ok().map(|r| r.level(faction))
    }

    /// 查询两个阵营间的基础关系。
    pub fn faction_relation(&self, a: &FactionId, b: &FactionId) -> FactionRelationType {
        self.faction_relation_table.get_relation(a, b)
    }

    /// 获取实体所属的所有阵营 ID 列表。
    ///
    /// 返回 `None` 如果实体没有 FactionMembership 组件。
    pub fn factions(&self, entity: Entity) -> Option<Vec<FactionId>> {
        self.faction_memberships
            .get(entity)
            .ok()
            .map(|m| m.factions.clone())
    }
}
