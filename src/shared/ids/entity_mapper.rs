//! EntityMapper — Domain 层隔离 Entity 的双向映射。
//!
//! 提供 `业务 ID ↔ Entity` 的双向映射，确保 Domain 层代码不直接依赖 Entity。
//!
//! # 设计原则
//!
//! - Domain 层只使用业务 ID（如 `UnitId`、`AbilityId`）
//! - Infrastructure 层维护 `EntityMapper` 负责双向映射
//! - 业务代码通过 `EntityMapper` 获取 Entity，不直接持有 Entity
//!
//! # 使用
//!
//! ```ignore
//! use crate::shared::ids::entity_mapper::EntityMapper;
//!
//! let mut mapper = EntityMapper::new();
//! mapper.register(unit_id, entity);
//!
//! // Domain 层使用
//! if let Some(entity) = mapper.get_entity(&unit_id) {
//!     // 通过 Entity 执行 ECS 操作
//! }
//!
//! // 从 Entity 反查
//! if let Some(id) = mapper.get_id(&entity) {
//!     // 获取业务 ID
//! }
//! ```

use bevy::prelude::*;
use std::collections::HashMap;

/// Entity 双向映射 Resource。
///
/// 维护 `业务 ID ↔ Entity` 的双向映射，确保 Domain 层代码不直接依赖 Entity。
///
/// # 类型参数
///
/// - `ID`: 业务 ID 类型（如 `UnitId`、`AbilityId`），必须实现 `Eq + Hash + Clone`
///
/// # 使用场景
///
/// - 战斗系统：`UnitId ↔ Entity`
/// - 技能系统：`AbilityId ↔ Entity`（如果需要）
/// - 通用场景：任何需要在 Domain 层和 ECS 层之间转换的 ID
///
/// # 注意事项
///
/// - EntityMapper 是 Resource，通过 `Res<EntityMapper<ID>>` 或 `ResMut<EntityMapper<ID>>` 访问
/// - 每种 ID 类型需要独立的 EntityMapper 实例
/// - Entity 重建后（存档加载、场景重载）需要重新填充映射
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct EntityMapper<ID: Eq + std::hash::Hash + Clone + 'static = UnitId> {
    /// Entity → ID 正向映射
    entity_to_id: HashMap<Entity, ID>,
    /// ID → Entity 反向映射
    id_to_entity: HashMap<ID, Entity>,
}

impl<ID: Eq + std::hash::Hash + Clone + 'static> EntityMapper<ID> {
    /// 创建空的映射器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个映射关系。
    ///
    /// 如果已存在旧映射，会被覆盖。
    pub fn register(&mut self, id: ID, entity: Entity) {
        self.id_to_entity.insert(id.clone(), entity);
        self.entity_to_id.insert(entity, id);
    }

    /// 注销一个 Entity 的所有映射。
    pub fn unregister_entity(&mut self, entity: &Entity) -> Option<ID> {
        if let Some(id) = self.entity_to_id.remove(entity) {
            self.id_to_entity.remove(&id);
            Some(id)
        } else {
            None
        }
    }

    /// 注销一个 ID 的所有映射。
    pub fn unregister_id(&mut self, id: &ID) -> Option<Entity> {
        if let Some(entity) = self.id_to_entity.remove(id) {
            self.entity_to_id.remove(&entity);
            Some(entity)
        } else {
            None
        }
    }

    /// 通过 Entity 查询业务 ID。
    pub fn get_id(&self, entity: &Entity) -> Option<&ID> {
        self.entity_to_id.get(entity)
    }

    /// 通过业务 ID 查询 Entity。
    pub fn get_entity(&self, id: &ID) -> Option<&Entity> {
        self.id_to_entity.get(id)
    }

    /// 检查 Entity 是否已注册。
    pub fn contains_entity(&self, entity: &Entity) -> bool {
        self.entity_to_id.contains_key(entity)
    }

    /// 检查 ID 是否已注册。
    pub fn contains_id(&self, id: &ID) -> bool {
        self.id_to_entity.contains_key(id)
    }

    /// 映射是否为空。
    pub fn is_empty(&self) -> bool {
        self.entity_to_id.is_empty()
    }

    /// 映射数量。
    pub fn len(&self) -> usize {
        self.entity_to_id.len()
    }

    /// 清空所有映射。
    pub fn clear(&mut self) {
        self.entity_to_id.clear();
        self.id_to_entity.clear();
    }

    /// 获取所有 Entity 的迭代器。
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entity_to_id.keys()
    }

    /// 获取所有 ID 的迭代器。
    pub fn ids(&self) -> impl Iterator<Item = &ID> {
        self.id_to_entity.keys()
    }
}

/// 默认的 UnitId 类型别名。
///
/// 用于战斗系统的 `UnitId ↔ Entity` 映射。
pub type UnitEntityMapper = EntityMapper<UnitId>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_mapper_register_and_query() {
        let mut mapper = EntityMapper::<String>::new();
        let entity = Entity::from_raw(42);
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
        let entity = Entity::from_raw(42);
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
        let entity = Entity::from_raw(42);
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
        let entity1 = Entity::from_raw(42);
        let entity2 = Entity::from_raw(43);
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

        mapper.register("a".to_string(), Entity::from_raw(1));
        mapper.register("b".to_string(), Entity::from_raw(2));
        assert_eq!(mapper.len(), 2);

        mapper.unregister_entity(&Entity::from_raw(1));
        assert_eq!(mapper.len(), 1);
    }

    #[test]
    fn entity_mapper_iterators() {
        let mut mapper = EntityMapper::<String>::new();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);

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
        mapper.register("a".to_string(), Entity::from_raw(1));
        mapper.register("b".to_string(), Entity::from_raw(2));

        mapper.clear();
        assert!(mapper.is_empty());
    }
}
