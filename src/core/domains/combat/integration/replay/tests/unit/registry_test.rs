//! BattleUnitRegistry 单元测试
//!
//! 验证双向映射的正确性，不启动 Bevy App。
//!
//! Test IDs:
//! - REG-001: registry_bidirectional_mapping
//! - REG-002: registry_is_empty_after_clear
//! - REG-003: registry_unknown_entity_returns_none
//! - REG-004: default_registry_is_empty

use crate::core::domains::combat::integration::replay::registry::{
    BattleUnitId, BattleUnitRegistry,
};
use bevy::prelude::Entity;

#[test]
fn registry_bidirectional_mapping() {
    // Given: 两个实体和对应的 BattleUnitId
    let mut registry = BattleUnitRegistry::default();
    let e1 = Entity::from_raw_u32(1).unwrap();
    let e2 = Entity::from_raw_u32(2).unwrap();
    let id1 = BattleUnitId::new("bu:player:0");
    let id2 = BattleUnitId::new("bu:enemy:0");

    // When: 注册两个实体
    registry.register(e1, id1.clone());
    registry.register(e2, id2.clone());

    // Then: 正向+反向映射均正确，get_entity_by_str 也工作
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
    // Given: 有一个实体的注册表
    let mut registry = BattleUnitRegistry::default();
    let e1 = Entity::from_raw_u32(1).unwrap();
    registry.register(e1, BattleUnitId::new("bu:player:0"));
    assert!(!registry.is_empty());

    // When: 清理注册表
    registry.clear();

    // Then: 注册表为空
    assert!(registry.is_empty());
}

#[test]
fn registry_unknown_entity_returns_none() {
    // Given: 空的注册表
    let registry = BattleUnitRegistry::default();

    // When/Then: 查询不存在的实体返回 None
    let unknown = Entity::from_raw_u32(999).unwrap();
    assert!(registry.get_id(&unknown).is_none());
    assert!(registry.get_entity_by_str("bu:nobody:0").is_none());
}

#[test]
fn default_registry_is_empty() {
    // Given/When: 默认构造的注册表
    let registry = BattleUnitRegistry::default();

    // Then: 为空
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}
