//! FactionFacade — Faction 域读写的业务语义 API。
//!
//! 所有 Faction 域组件（FactionMembership、Reputation、KeyCharacter 等）的
//! 字段访问都在此文件中完成。Systems 通过 ReadFacade / WriteFacade 交互，
//! 永远不直接访问 Faction 组件的字段。
//!
//! # 职责边界
//!
//! - ✅ 封装对 FactionMembership / Reputation / KeyCharacter / FactionRelationTable 的读写
//! - ✅ 提供 spawn_faction_member 快捷创建
//! - 🟥 禁止：在此模块之外直接 import FactionMembership / Reputation / KeyCharacter 进行字段访问

use bevy::prelude::*;

use crate::core::domains::faction::components::{
    FactionId, FactionMembership, FactionRelationTable, FactionRelationType, KeyCharacter,
    Reputation, ReputationLevel,
};

// ─── 只读操作 ────────────────────────────────────────────────────────────

/// Faction 域只读查询接口。
///
/// 外部通过此结构体的 static 方法读取 Faction 域数据。
/// 所有方法接受 `&World`，返回只读引用或拷贝值。
pub struct FactionReadFacade;

impl FactionReadFacade {
    /// 获取实体的 FactionMembership 组件引用。
    pub fn faction_membership(world: &World, entity: Entity) -> Option<&FactionMembership> {
        world.get::<FactionMembership>(entity)
    }

    /// 获取实体的 Reputation 组件引用。
    pub fn reputation(world: &World, entity: Entity) -> Option<&Reputation> {
        world.get::<Reputation>(entity)
    }

    /// 检查实体是否拥有 KeyCharacter 标记。
    pub fn is_key_character(world: &World, entity: Entity) -> bool {
        world.get::<KeyCharacter>(entity).is_some()
    }

    /// 获取 FactionRelationTable 资源引用。
    pub fn faction_relation_table(world: &World) -> Option<&FactionRelationTable> {
        world.get_resource::<FactionRelationTable>()
    }

    /// 检查实体是否属于指定阵营。
    ///
    /// 返回 `None` 如果实体没有 FactionMembership 组件。
    pub fn has_faction(world: &World, entity: Entity, faction: &FactionId) -> Option<bool> {
        world
            .get::<FactionMembership>(entity)
            .map(|m| m.is_member(faction))
    }

    /// 获取实体在指定阵营的声望值。
    ///
    /// 返回 `None` 如果实体没有 Reputation 组件。
    pub fn get_reputation_value(world: &World, entity: Entity, faction: &FactionId) -> Option<i32> {
        world.get::<Reputation>(entity).map(|r| r.get(faction))
    }

    /// 获取实体在指定阵营的声望等级。
    ///
    /// 返回 `None` 如果实体没有 Reputation 组件。
    pub fn get_reputation_level(
        world: &World,
        entity: Entity,
        faction: &FactionId,
    ) -> Option<ReputationLevel> {
        world.get::<Reputation>(entity).map(|r| r.level(faction))
    }

    /// 查询两个阵营间的基础关系。
    ///
    /// 返回 `None` 如果 FactionRelationTable 资源不存在。
    pub fn get_faction_relation(
        world: &World,
        a: &FactionId,
        b: &FactionId,
    ) -> Option<FactionRelationType> {
        world
            .get_resource::<FactionRelationTable>()
            .map(|table| table.get_relation(a, b))
    }

    /// 获取实体所属的所有阵营 ID 列表。
    ///
    /// 返回 `None` 如果实体没有 FactionMembership 组件。
    pub fn get_factions(world: &World, entity: Entity) -> Option<Vec<FactionId>> {
        world
            .get::<FactionMembership>(entity)
            .map(|m| m.factions.clone())
    }
}

// ─── 写操作 ──────────────────────────────────────────────────────────────

/// Faction 域写入接口。
///
/// 外部通过此结构体的 static 方法修改 Faction 域数据。
/// 方法直接修改组件状态（通过 `&mut World` 或 `Commands`），
/// 不触发 Faction 域的 Observer 事件循环——调用方如需通知其他系统应手动触发事件。
pub struct FactionWriteFacade;

impl FactionWriteFacade {
    /// 为实体加入指定阵营。
    ///
    /// 直接调用 `FactionMembership::join()`。重复加入同一阵营为 no-op。
    /// 不触发 `FactionJoined` 事件——调用方如需通知其他系统应自行触发。
    pub fn join_faction(world: &mut World, entity: Entity, faction: impl Into<FactionId>) {
        if let Some(mut membership) = world.get_mut::<FactionMembership>(entity) {
            membership.join(faction);
        }
    }

    /// 为实体离开指定阵营。
    ///
    /// 直接调用 `FactionMembership::leave()`。不是阵营成员则为 no-op。
    /// 不触发 `FactionLeft` 事件——调用方如需通知其他系统应自行触发。
    pub fn leave_faction(world: &mut World, entity: Entity, faction: &FactionId) {
        if let Some(mut membership) = world.get_mut::<FactionMembership>(entity) {
            membership.leave(faction);
        }
    }

    /// 设置实体在指定阵营的声望值。
    ///
    /// 直接调用 `Reputation::set()`。值会被 clamp 到 [-100, +100]。
    /// 不触发 `ReputationChanged` 事件——调用方如需通知其他系统应自行触发。
    pub fn set_reputation(world: &mut World, entity: Entity, faction: FactionId, value: i32) {
        if let Some(mut reputation) = world.get_mut::<Reputation>(entity) {
            reputation.set(faction, value);
        }
    }

    /// 修改实体在指定阵营的声望值（delta 可为负）。
    ///
    /// 直接调用 `Reputation::modify()`。结果值会被 clamp 到 [-100, +100]。
    /// 不触发 `ReputationChanged` 事件——调用方如需通知其他系统应自行触发。
    pub fn modify_reputation(world: &mut World, entity: Entity, faction: &FactionId, delta: i32) {
        if let Some(mut reputation) = world.get_mut::<Reputation>(entity) {
            reputation.modify(faction, delta);
        }
    }

    /// 设置两个阵营间的基关系。
    ///
    /// 直接调用 `FactionRelationTable::set_relation()`。自动维护对称性。
    /// 不触发 `FactionRelationChanged` 事件——调用方如需通知其他系统应自行触发。
    pub fn set_faction_relation(
        world: &mut World,
        a: FactionId,
        b: FactionId,
        relation: FactionRelationType,
    ) {
        if let Some(mut table) = world.get_resource_mut::<FactionRelationTable>() {
            table.set_relation(a, b, relation);
        }
    }

    /// 生成一个基本的阵营成员实体。
    ///
    /// 创建的实体包含：
    /// - FactionMembership（指定阵营）
    /// - Reputation（空，所有阵营默认 0/Neutral）
    pub fn spawn_faction_member(commands: &mut Commands, factions: Vec<FactionId>) -> Entity {
        let mut membership = FactionMembership::new();
        for f in factions {
            membership.join(f);
        }
        commands.spawn((membership, Reputation::new())).id()
    }
}
