//! PartyQueryParam — Bevy SystemParam，封装所有 Party 域组件/资源查询。
//!
//! Systems 通过此 param 读取队伍数据，完全不知道 `Party` / `BondState` /
//! `PartyMarker` 资源/组件的存在细节。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     party_query: PartyQueryParam,
//!     // ...
//! ) {
//!     let party = party_query.get_party();
//!     if party_query.is_in_party(entity) {
//!         // 处理队伍成员
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 只提供只读查询——可变操作通过 `PartyWriteFacade` 完成
//! - 不包装 `Commands`——调用方传入以保持语义清晰
//! - 使用 `Res` 直接包裹 Resource，避免每次方法调用从 World 查询的性能开销

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::domains::party::components::{BondDefId, BondState, Party, PartyMarker};

/// 队伍查询 SystemParam — 封装所有 Party 域组件/资源查询。
///
/// System 签名中使用此类型替代裸 `Res<Party>` + `Res<BondState>` + `Query<&PartyMarker>`。
#[derive(SystemParam)]
pub struct PartyQueryParam<'w, 's> {
    /// PartyMarker 实体查询
    party_marker_query: Query<'w, 's, Entity, With<PartyMarker>>,
    /// Party 资源
    party: Res<'w, Party>,
    /// 羁绊状态资源
    bond_state: Res<'w, BondState>,
}

impl<'w, 's> PartyQueryParam<'w, 's> {
    /// 获取 Party Resource。
    ///
    /// # Returns
    /// - `&Party` — Party 资源引用
    pub fn get_party(&self) -> &Party {
        &self.party
    }

    /// 获取 BondState Resource。
    ///
    /// # Returns
    /// - `&BondState` — 羁绊状态资源引用
    pub fn get_bond_state(&self) -> &BondState {
        &self.bond_state
    }

    /// 检查指定实体是否拥有 PartyMarker 组件。
    ///
    /// # Returns
    /// - `true` — 实体拥有 `PartyMarker` 组件
    /// - `false` — 实体无该组件
    pub fn has_party_marker(&self, entity: Entity) -> bool {
        self.party_marker_query.get(entity).is_ok()
    }

    /// 获取所有拥有 PartyMarker 的实体。
    ///
    /// # Returns
    /// - 所有标记为 Party 实体的列表
    pub fn party_entities(&self) -> Vec<Entity> {
        self.party_marker_query.iter().collect()
    }

    /// 检查指定实体是否在队伍中（活跃或预备）。
    ///
    /// # Returns
    /// - `true` — 实体是活跃成员或预备队员
    /// - `false` — 实体不在队伍中
    pub fn is_in_party(&self, entity: Entity) -> bool {
        self.party.members.iter().any(|m| m.entity == entity)
            || self.party.reserve_members.contains(&entity)
    }

    /// 检查指定实体是否为活跃成员。
    ///
    /// # Returns
    /// - `true` — 实体在活跃成员列表中且 `is_active == true`
    /// - `false` — 实体不是活跃成员
    pub fn is_member_active(&self, entity: Entity) -> bool {
        self.party
            .members
            .iter()
            .any(|m| m.entity == entity && m.is_active)
    }

    /// 检查指定羁绊是否已激活。
    ///
    /// # Returns
    /// - `true` — 该羁绊在激活列表中
    /// - `false` — 未激活
    pub fn is_bond_active(&self, bond_id: &BondDefId) -> bool {
        self.bond_state
            .active_bonds
            .iter()
            .any(|ab| ab.bond_id == *bond_id)
    }

    /// 获取当前激活的羁绊数量。
    ///
    /// # Returns
    /// - 激活羁绊数
    pub fn get_active_bond_count(&self) -> usize {
        self.bond_state.active_bonds.len()
    }
}
