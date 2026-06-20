//! EntityMapper 单元测试
//!
//! 验证 Entity ↔ 业务 ID 的双向映射操作：注册、查询、覆盖、迭代、清理。

use bevy::prelude::Entity;

use crate::shared::ids::mapping::EntityMapper;

#[test]
fn entity_mapper_register_and_query() {
    let mut mapper = EntityMapper::<String>::new();
    let entity = Entity::from_bits(42);
    let id = "unit_001".to_string();

    mapper.register(id.clone(), entity);

    assert_eq!(mapper.get_id(&entity), Some(&id));
    assert_eq!(mapper.get_entity(&id), Some(&entity));
    assert!(mapper.contains_entity(&entity));
    assert!(mapper.contains_id(&id));
}

#[test]
fn entity_mapper_unregister_entity() {
    let mut mapper = EntityMapper::<String>::new();
    let entity = Entity::from_bits(42);
    let id = "unit_001".to_string();

    mapper.register(id.clone(), entity);
    let removed = mapper.unregister_entity(&entity);

    assert_eq!(removed, Some(id));
    assert!(!mapper.contains_entity(&entity));
    assert!(mapper.is_empty());
}

#[test]
fn entity_mapper_unregister_id() {
    let mut mapper = EntityMapper::<String>::new();
    let entity = Entity::from_bits(42);
    let id = "unit_001".to_string();

    mapper.register(id.clone(), entity);
    let removed = mapper.unregister_id(&id);

    assert_eq!(removed, Some(entity));
    assert!(!mapper.contains_id(&id));
    assert!(mapper.is_empty());
}

#[test]
fn entity_mapper_overwrite() {
    let mut mapper = EntityMapper::<String>::new();
    let entity1 = Entity::from_bits(42);
    let entity2 = Entity::from_bits(43);
    let id = "unit_001".to_string();

    mapper.register(id.clone(), entity1);
    mapper.register(id.clone(), entity2);

    assert_eq!(mapper.get_entity(&id), Some(&entity2));
    assert!(!mapper.contains_entity(&entity1));
}

#[test]
fn entity_mapper_len() {
    let mut mapper = EntityMapper::<String>::new();
    assert_eq!(mapper.len(), 0);

    mapper.register("a".to_string(), Entity::from_bits(1));
    mapper.register("b".to_string(), Entity::from_bits(2));
    assert_eq!(mapper.len(), 2);

    mapper.unregister_entity(&Entity::from_bits(1));
    assert_eq!(mapper.len(), 1);
}

#[test]
fn entity_mapper_iterators() {
    let mut mapper = EntityMapper::<String>::new();
    let e1 = Entity::from_bits(1);
    let e2 = Entity::from_bits(2);

    mapper.register("a".to_string(), e1);
    mapper.register("b".to_string(), e2);

    let entities: Vec<_> = mapper.entities().copied().collect();
    assert!(entities.contains(&e1));
    assert!(entities.contains(&e2));

    let ids: Vec<_> = mapper.ids().cloned().collect();
    assert!(ids.contains(&"a".to_string()));
    assert!(ids.contains(&"b".to_string()));
}

#[test]
fn entity_mapper_clear() {
    let mut mapper = EntityMapper::<String>::new();
    mapper.register("a".to_string(), Entity::from_bits(1));
    mapper.register("b".to_string(), Entity::from_bits(2));

    mapper.clear();
    assert!(mapper.is_empty());
}
