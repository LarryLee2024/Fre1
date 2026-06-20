//! EntityMapper<BattleUnitId> 单元测试
//!
//! 验证 BattleUnitId 通过 EntityMapper 的双向映射正确性。
//!
//! Test IDs:
//! - REG-001: bidirectional_mapping
//! - REG-002: is_empty_after_clear
//! - REG-003: unknown_entity_returns_none
//! - REG-004: default_is_empty

use bevy::prelude::Entity;

use crate::shared::ids::BattleUnitId;
use crate::shared::ids::mapping::EntityMapper;

#[test]
fn bidirectional_mapping() {
    // Given: 两个实体和对应的 BattleUnitId
    let mut mapper = EntityMapper::<BattleUnitId>::new();
    let e1 = Entity::from_raw_u32(1).unwrap();
    let e2 = Entity::from_raw_u32(2).unwrap();
    let id1 = BattleUnitId::new("bu:0:0");
    let id2 = BattleUnitId::new("bu:1:0");

    // When: 注册两个实体
    mapper.register(id1.clone(), e1);
    mapper.register(id2.clone(), e2);

    // Then: 正向+反向映射均正确
    assert_eq!(mapper.get_id(&e1), Some(&id1));
    assert_eq!(mapper.get_id(&e2), Some(&id2));
    assert_eq!(mapper.get_entity(&id1), Some(&e1));
    assert_eq!(mapper.get_entity(&id2), Some(&e2));
    assert_eq!(mapper.len(), 2);
}

#[test]
fn is_empty_after_clear() {
    // Given: 有实体注册的映射器
    let mut mapper = EntityMapper::<BattleUnitId>::new();
    let e1 = Entity::from_raw_u32(1).unwrap();
    mapper.register(BattleUnitId::new("bu:0:0"), e1);
    assert!(!mapper.is_empty());

    // When: 清理
    mapper.clear();

    // Then: 为空
    assert!(mapper.is_empty());
}

#[test]
fn unknown_entity_returns_none() {
    // Given: 空的映射器
    let mapper = EntityMapper::<BattleUnitId>::new();

    // When/Then: 查询不存在的实体返回 None
    let unknown = Entity::from_raw_u32(999).unwrap();
    assert!(mapper.get_id(&unknown).is_none());
    assert!(
        mapper
            .get_entity(&BattleUnitId::new("bu:nobody:0"))
            .is_none()
    );
}

#[test]
fn default_is_empty() {
    // Given/When: 默认构造
    let mapper = EntityMapper::<BattleUnitId>::new();

    // Then: 为空
    assert!(mapper.is_empty());
    assert_eq!(mapper.len(), 0);
}
