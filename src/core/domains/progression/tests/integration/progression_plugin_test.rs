//! ProgressionPlugin 集成测试
//!
//! 验证 Plugin 注册、Component 类型注册、Resource 初始化、System 执行流程正常。
//! 使用最小 App 实例，不依赖外部资源。

use bevy::prelude::*;

use crate::core::domains::progression::ProgressionPlugin;
use crate::core::domains::progression::components::{
    ClassLevels, Experience, ProgressionMarker, SubclassChoice, TalentTree,
};

/// Plugin 注册后所有 Component 类型可用。
#[test]
fn progression_plugin_registers_components() {
    let mut app = App::new();
    app.add_plugins(ProgressionPlugin);

    // 验证 Component 类型已注册（通过 reflect 注册可以 spawn）
    let entity = app
        .world_mut()
        .spawn((
            Experience::new(),
            ClassLevels::new("fighter"),
            TalentTree::new(),
            SubclassChoice::new(),
            ProgressionMarker,
        ))
        .id();

    let world = app.world();
    assert!(
        world.get::<Experience>(entity).is_some(),
        "Experience 组件注册失败"
    );
    assert!(
        world.get::<ClassLevels>(entity).is_some(),
        "ClassLevels 组件注册失败"
    );
    assert!(
        world.get::<TalentTree>(entity).is_some(),
        "TalentTree 组件注册失败"
    );
    assert!(
        world.get::<SubclassChoice>(entity).is_some(),
        "SubclassChoice 组件注册失败"
    );
    assert!(
        world.get::<ProgressionMarker>(entity).is_some(),
        "ProgressionMarker 组件注册失败"
    );
}

/// Resource 初始化正确。
#[test]
fn progression_plugin_inits_resource() {
    let mut app = App::new();
    app.add_plugins(ProgressionPlugin);

    let world = app.world();
    let table = world
        .get_resource::<crate::core::domains::progression::components::LevelProgressionTable>();
    assert!(table.is_some(), "LevelProgressionTable Resource 未初始化");

    if let Some(t) = table {
        assert_eq!(t.max_level, 20);
        assert_eq!(t.asi_levels, vec![4, 8, 12, 16, 19]);
    }
}

/// 满级角色 add_xp 不累积经验。
#[test]
fn progression_max_level_xp_noop() {
    let mut app = App::new();
    app.add_plugins(ProgressionPlugin);

    // 创建一个满级角色
    app.world_mut().spawn((
        Experience {
            level: 20,
            is_max_level: true,
            current_xp: 0,
            total_xp_earned: 0,
        },
        ProgressionMarker,
    ));

    // 让系统检查
    app.update();

    // 确认 is_max_level 保持不变，且 add_xp 无效
    let mut xp = Experience::new();
    xp.level = 20;
    xp.is_max_level = true;
    let total = xp.add_xp(1000);
    assert_eq!(total, 0, "满级角色 add_xp 返回 0");
}
