// 场景快照工具：基于 DynamicSceneBuilder 的 World 序列化
// 用于战斗回放、存档、调试快照等场景

use bevy::prelude::*;
use bevy::scene::DynamicSceneBuilder;

/// 将指定 Entity 列表序列化为 RON 字符串
/// 仅序列化已注册 Reflect 的 Component
pub fn save_snapshot(world: &mut World, entities: &[Entity]) -> Option<String> {
    let registry = world.resource::<AppTypeRegistry>();
    let type_registry = registry.read();
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(entities.iter().copied())
        .build();
    scene.serialize(&type_registry).ok()
}

/// 将 World 中所有 Entity 序列化为 RON 字符串
pub fn save_full_snapshot(world: &mut World) -> Option<String> {
    // 通过 QueryState 收集所有 Entity ID
    let mut query_state = world.query::<Entity>();
    let all_entities: Vec<Entity> = query_state.iter(world).collect();
    drop(query_state);

    let registry = world.resource::<AppTypeRegistry>();
    let type_registry = registry.read();
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(all_entities.iter().copied())
        .build();
    scene.serialize(&type_registry).ok()
}
