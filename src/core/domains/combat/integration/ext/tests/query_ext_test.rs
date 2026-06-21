//! QueryExt 桩实现的测试。
//!
//! 这些测试验证 QueryExt 方法可以从系统上下文中调用。

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use crate::core::domains::combat::integration::ext::QueryExt;

/// 测试实体的标记组件。
#[derive(Component)]
struct _Health(u32);

/// 在 Query 上调用 alive() 的系统 — 验证 trait 可编译。
fn alive_system(query: Query<&_Health>) {
    let _alive: Vec<Entity> = query.alive().collect();
}

/// 在 Query 上调用 hostile_to() 的系统 — 验证 trait 可编译。
fn hostile_system(query: Query<&_Health>) {
    let _hostile: Vec<Entity> = query.hostile_to("enemy").collect();
}

#[test]
fn alive_method_compiles_in_system() {
    let mut app = App::new();
    app.add_systems(Update, alive_system);
    app.update();

    // Stub always returns empty, so this just verifies no crash.
}

#[test]
fn hostile_to_method_compiles_in_system() {
    let mut app = App::new();
    app.add_systems(Update, hostile_system);
    app.update();

    // Stub always returns empty, so this just verifies no crash.
}

#[test]
fn both_filters_return_empty() {
    let mut app = App::new();
    // Spawn an entity to verify the stub doesn't interact with real data.
    app.world_mut().spawn(_Health(100));

    app.add_systems(Update, |query: Query<&_Health>| {
        let alive: Vec<_> = query.alive().collect();
        assert!(alive.is_empty(), "stub alive() should return empty iterator");

        let hostile: Vec<_> = query.hostile_to("faction").collect();
        assert!(hostile.is_empty(), "stub hostile_to() should return empty iterator");
    });
    app.update();
}
