//! PartyReadFacade + PartyWriteFacade — Party 域组件/资源读写入口。
//!
//! # ReadFacade — 只读查询 API
//!
//! 通过 `&World` 提供对 Party 域 ECS 组件的不可变访问。
//! 所有方法为静态函数，可在任何能访问 `&World` 的地方使用：
//! - Bevy Systems 中通过 `system_param` 获取 `&World`
//! - 测试代码中直接使用
//!
//! # WriteFacade — 可变操作 API
//!
//! 提供对 Party 域 Resource/Component 的修改操作，通过 `&mut World` 立即执行。
//! 不包含业务校验——校验应在调用前通过 domain rules 完成。
//!
//! # 设计
//!
//! - 所有方法不发射事件（Event）——事件发射由调用方（System）负责
//! - WriteFacade 仅执行原始数据变更，不含业务校验逻辑
//! - 校验应在调用 WriteFacade 之前通过 domain rules 完成

use bevy::prelude::*;

use crate::core::domains::party::components::{
    ActiveBond, BondDef, BondDefId, BondState, FormationType, Party, PartyMarker, PartyMember,
};

// ─── PartyReadFacade ──────────────────────────────────────────────────────

/// ReadFacade — 只读查询 API
///
/// 提供对 Party 域 ECS 组件（Party Resource, BondState Resource,
/// PartyMarker Component）的只读访问。
/// 所有方法通过 `&World` 查询资源或组件，不包含业务逻辑。
pub struct PartyReadFacade;

impl PartyReadFacade {
    /// 获取 Party Resource。
    ///
    /// # Returns
    /// - `Some(&Party)` — 如果 `Party` Resource 已注册
    /// - `None` — 如果 Resource 不存在
    pub fn get_party(world: &World) -> Option<&Party> {
        world.get_resource::<Party>()
    }

    /// 获取 BondState Resource。
    ///
    /// # Returns
    /// - `Some(&BondState)` — 如果 `BondState` Resource 已注册
    /// - `None` — 如果 Resource 不存在
    pub fn get_bond_state(world: &World) -> Option<&BondState> {
        world.get_resource::<BondState>()
    }

    /// 获取当前阵型类型。
    ///
    /// # Returns
    /// - `Some(FormationType)` — 如果 Party 资源存在
    /// - `None` — 如果 Party 资源不存在
    pub fn get_formation(world: &World) -> Option<FormationType> {
        world.get_resource::<Party>().map(|p| p.formation.clone())
    }

    /// 获取当前选中成员索引。
    ///
    /// # Returns
    /// - `Some(usize)` — 有选中成员
    /// - `None` — 无选中成员或 Party 资源不存在
    pub fn get_active_member_index(world: &World) -> Option<usize> {
        world.get_resource::<Party>().and_then(|p| p.active_member)
    }

    /// 检查指定实体是否在队伍中（活跃或预备）。
    ///
    /// # Returns
    /// - `true` — 实体是活跃成员或预备队员
    /// - `false` — 实体不在队伍中，或 Party 资源不存在
    pub fn is_in_party(world: &World, entity: Entity) -> bool {
        world.get_resource::<Party>().is_some_and(|p| {
            p.members.iter().any(|m| m.entity == entity) || p.reserve_members.contains(&entity)
        })
    }

    /// 检查指定实体是否为活跃成员。
    ///
    /// # Returns
    /// - `true` — 实体在活跃成员列表中且 `is_active == true`
    /// - `false` — 实体不是活跃成员，或 Party 资源不存在
    pub fn is_member_active(world: &World, entity: Entity) -> bool {
        world
            .get_resource::<Party>()
            .is_some_and(|p| p.members.iter().any(|m| m.entity == entity && m.is_active))
    }

    /// 检查实体是否拥有 PartyMarker 组件。
    ///
    /// # Returns
    /// - `true` — 实体拥有 `PartyMarker` 组件
    /// - `false` — 实体不存在或无该组件
    pub fn has_party_marker(world: &World, entity: Entity) -> bool {
        world.get::<PartyMarker>(entity).is_some()
    }

    /// 检查指定羁绊是否已激活。
    ///
    /// # Returns
    /// - `true` — 该羁绊在激活列表中
    /// - `false` — 未激活或 BondState 资源不存在
    pub fn is_bond_active(world: &World, bond_id: &BondDefId) -> bool {
        world
            .get_resource::<BondState>()
            .is_some_and(|b| b.active_bonds.iter().any(|ab| ab.bond_id == *bond_id))
    }

    /// 获取当前激活的羁绊数量。
    ///
    /// # Returns
    /// - 激活羁绊数（BondState 不存在时返回 0）
    pub fn get_active_bond_count(world: &World) -> usize {
        world
            .get_resource::<BondState>()
            .map_or(0, |b| b.active_bonds.len())
    }
}

// ─── PartyWriteFacade ──────────────────────────────────────────────────────

/// WriteFacade — 可变操作 API
///
/// 提供对 Party 域 Resource/Component 的修改操作。
/// 不包含业务校验——校验应在调用前通过 domain rules 完成。
pub struct PartyWriteFacade;

impl PartyWriteFacade {
    // ── &mut World 方法：立即执行 ────────────────────────────────────

    /// 设置队伍阵型。
    ///
    /// # WriteFacade: 立即设置阵型
    pub fn set_formation(world: &mut World, formation: FormationType) {
        if let Some(mut party) = world.get_resource_mut::<Party>() {
            party.formation = formation;
        }
    }

    /// 设置当前选中成员索引。
    ///
    /// # WriteFacade: 立即设置选中成员
    pub fn set_active_member_index(world: &mut World, index: Option<usize>) {
        if let Some(mut party) = world.get_resource_mut::<Party>() {
            party.active_member = index;
        }
    }

    /// 添加一名活跃成员到队伍。
    ///
    /// 不做重复性检查或容量校验——调用方应通过 domain rules 校验。
    ///
    /// # WriteFacade: 立即添加活跃成员
    pub fn add_member(world: &mut World, member: PartyMember) {
        if let Some(mut party) = world.get_resource_mut::<Party>() {
            party.members.push(member);
        }
    }

    /// 从队伍中移除指定实体（活跃成员）。
    ///
    /// 如果实体不在成员列表中则无操作。
    ///
    /// # WriteFacade: 立即移除活跃成员
    pub fn remove_member(world: &mut World, entity: Entity) {
        if let Some(mut party) = world.get_resource_mut::<Party>() {
            party.members.retain(|m| m.entity != entity);
        }
    }

    /// 添加一名预备队员。
    ///
    /// 如果实体已在预备列表则跳过。
    ///
    /// # WriteFacade: 立即添加预备队员
    pub fn add_reserve_member(world: &mut World, entity: Entity) {
        if let Some(mut party) = world.get_resource_mut::<Party>()
            && !party.reserve_members.contains(&entity)
        {
            party.reserve_members.push(entity);
        }
    }

    /// 移除一名预备队员。
    ///
    /// 如果实体不在预备列表则无操作。
    ///
    /// # WriteFacade: 立即移除预备队员
    pub fn remove_reserve_member(world: &mut World, entity: Entity) {
        if let Some(mut party) = world.get_resource_mut::<Party>() {
            party.reserve_members.retain(|e| *e != entity);
        }
    }

    /// 激活一个羁绊。
    ///
    /// 不做重复激活检查——调用方应通过 domain rules 校验。
    ///
    /// # WriteFacade: 立即激活羁绊
    pub fn activate_bond(world: &mut World, bond: ActiveBond) {
        if let Some(mut bond_state) = world.get_resource_mut::<BondState>() {
            bond_state.active_bonds.push(bond);
        }
    }

    /// 解除一个羁绊。
    ///
    /// # WriteFacade: 立即解除羁绊
    pub fn deactivate_bond(world: &mut World, bond_id: &BondDefId) {
        if let Some(mut bond_state) = world.get_resource_mut::<BondState>() {
            bond_state.active_bonds.retain(|b| b.bond_id != *bond_id);
        }
    }

    /// 注册一个羁绊模板定义。
    ///
    /// # WriteFacade: 立即注册羁绊模板
    pub fn register_bond_def(world: &mut World, def: BondDef) {
        let id = def.id.clone();
        if let Some(mut bond_state) = world.get_resource_mut::<BondState>() {
            bond_state.defs.insert(id, def);
        }
    }

    /// 添加 PartyMarker 组件到实体。
    ///
    /// # WriteFacade: 立即插入标记组件
    pub fn insert_party_marker(world: &mut World, entity: Entity) {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(PartyMarker);
        }
    }

    /// 从实体移除 PartyMarker 组件。
    ///
    /// # WriteFacade: 立即移除标记组件
    pub fn remove_party_marker(world: &mut World, entity: Entity) {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.remove::<PartyMarker>();
        }
    }
}
