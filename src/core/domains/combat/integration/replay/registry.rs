//! BattleUnitRegistry — 战场内 Entity ↔ 稳定 String ID 双向映射
//!
//! 用于 Replay 录制/回放时，将 Entity 句柄转换为 ReplayCommand 使用的 String 标识，
//! 以及回放时将 String 标识还原为 Entity。
//!
//! # 设计
//!
//! - `BattleUnitId` Component 直接挂在每个参战实体上，O(1) 查询
//! - `BattleUnitRegistry` Resource 提供双向 HashMap 查询（Entity ↔ BattleUnitId）
//! - ID 格式: `"bu:{team_index}:{unit_index}"` 如 `"bu:0:0"`
//!
//! # 生命周期
//!
//! 战斗开始时由 `start_recording_on_battle_begin` 创建并填充，
//! 战斗结束时由 `stop_recording_on_battle_end` 清理。
//!
//! 详见 ADR-048 §Module Design

use bevy::prelude::*;
use std::borrow::Borrow;
use std::collections::HashMap;

use crate::core::domains::combat::components::CombatParticipant;

/// 战场单位标识 Component。
///
/// 挂在每个参与战斗的实体上，提供 Entity 到稳定 String ID 的 O(1) 转换。
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleUnitId(pub String);

impl BattleUnitId {
    /// 创建新的战场单位标识。
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Borrow<str> for BattleUnitId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// 战场单位注册表 Resource。
///
/// 提供双向查询：Entity → BattleUnitId 和 BattleUnitId → Entity。
#[derive(Resource, Debug, Default)]
pub struct BattleUnitRegistry {
    /// Entity → BattleUnitId 正向映射
    entity_to_id: HashMap<Entity, BattleUnitId>,
    /// BattleUnitId → Entity 反向映射
    id_to_entity: HashMap<BattleUnitId, Entity>,
}

impl BattleUnitRegistry {
    /// 注册一个实体的战场标识。
    pub fn register(&mut self, entity: Entity, id: BattleUnitId) {
        self.id_to_entity.insert(id.clone(), entity);
        self.entity_to_id.insert(entity, id);
    }

    /// 通过 Entity 查询 BattleUnitId。
    pub fn get_id(&self, entity: &Entity) -> Option<&BattleUnitId> {
        self.entity_to_id.get(entity)
    }

    /// 通过 BattleUnitId 查询 Entity。
    pub fn get_entity(&self, id: &BattleUnitId) -> Option<&Entity> {
        self.id_to_entity.get(id)
    }

    /// 通过 String 查询 Entity（回放命令分发时使用）。
    pub fn get_entity_by_str(&self, id_str: &str) -> Option<&Entity> {
        let unit_id = BattleUnitId(id_str.to_string());
        self.id_to_entity.get(&unit_id)
    }

    /// 注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.entity_to_id.is_empty()
    }

    /// 清理所有注册。
    pub fn clear(&mut self) {
        self.entity_to_id.clear();
        self.id_to_entity.clear();
    }

    /// 获取所有 Entity 的迭代器。
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entity_to_id.keys()
    }

    /// 正向映射条目数。
    pub fn len(&self) -> usize {
        self.entity_to_id.len()
    }
}

/// 从 CombatParticipant 查询构建 BattleUnitRegistry。
///
/// 为每个 CombatParticipant 实体分配 BattleUnitId 并挂载 Component。
pub(crate) fn build_battle_unit_registry(
    participants: &Query<(Entity, &CombatParticipant)>,
    commands: &mut Commands,
) -> BattleUnitRegistry {
    let mut registry = BattleUnitRegistry::default();

    // 按 team 分组，为 each entity 分配格式为 "bu:{team_index}:{unit_index}" 的 ID
    let mut team_units: HashMap<String, Vec<Entity>> = HashMap::new();
    for (entity, participant) in participants.iter() {
        let team_id = participant.team_id.to_string();
        team_units.entry(team_id).or_default().push(entity);
    }

    for (team_id, entities) in &team_units {
        for (index, entity) in entities.iter().enumerate() {
            let id_str = format!("bu:{}:{}", team_id.replace(' ', "_"), index);
            let unit_id = BattleUnitId::new(&id_str);
            commands.entity(*entity).insert(unit_id.clone());
            registry.register(*entity, unit_id);
        }
    }

    registry
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn registry_bidirectional_mapping() {
        let mut registry = BattleUnitRegistry::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let id1 = BattleUnitId::new("bu:player:0");
        let id2 = BattleUnitId::new("bu:enemy:0");

        registry.register(e1, id1.clone());
        registry.register(e2, id2.clone());

        assert_eq!(registry.get_id(&e1), Some(&id1));
        assert_eq!(registry.get_id(&e2), Some(&id2));
        assert_eq!(registry.get_entity(&id1), Some(&e1));
        assert_eq!(registry.get_entity(&id2), Some(&e2));
        assert_eq!(registry.get_entity_by_str("bu:player:0"), Some(&e1));
        assert_eq!(registry.get_entity_by_str("bu:enemy:0"), Some(&e2));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn registry_is_empty_after_clear() {
        let mut registry = BattleUnitRegistry::default();
        let e1 = Entity::from_raw_u32(1).unwrap();
        registry.register(e1, BattleUnitId::new("bu:player:0"));
        assert!(!registry.is_empty());

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn registry_unknown_entity_returns_none() {
        let registry = BattleUnitRegistry::default();
        let unknown = Entity::from_raw_u32(999).unwrap();
        assert!(registry.get_id(&unknown).is_none());
        assert!(registry.get_entity_by_str("bu:nobody:0").is_none());
    }

    #[test]
    fn default_registry_is_empty() {
        let registry = BattleUnitRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
