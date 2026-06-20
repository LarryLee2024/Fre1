//! Integration: 内容管线 → AttributeRegistry 桥接系统
//!
//! 验证 `register_attributes_from_content` 系统正确地将 `LoadedAttributeDefs` 中的
//! `AttributeDefinition` 注册到 `AttributeRegistry`。
//!
//! 覆盖的场景:
//! - 正常注册: Primary + Resource 属性定义
//! - 空定义: 无属性定义时系统安全跳过
//! - 默认值越界: default_base_value 超出 [min, max] 范围被拒绝
//! - 重复 ID: 相同 AttributeId 的第二次推送不会重复注册
//! - Resource min < 0: Resource 类别的 min_value 不能为负

use bevy::prelude::*;

use crate::content::LoadedAttributeDefs;
use crate::core::capabilities::attribute::foundation::{
    AttributeCategory, AttributeDefinition, AttributeId,
};
use crate::core::capabilities::attribute::mechanism::AttributeRegistry;
use crate::core::capabilities::attribute::plugin::AttributePlugin;

/// 构造一个测试用属性定义。
fn make_attr(
    id: &str,
    category: AttributeCategory,
    default: f32,
    min: f32,
    max: f32,
) -> AttributeDefinition {
    AttributeDefinition {
        id: AttributeId::new(id),
        category,
        default_base_value: default,
        min_value: min,
        max_value: max,
        derived_dependencies: vec![],
        hidden: false,
    }
}

#[test]
fn bridge_registers_attributes_into_registry() {
    let mut app = App::new();
    app.init_resource::<LoadedAttributeDefs>();
    app.add_plugins(AttributePlugin);

    // 插入标准测试属性定义
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedAttributeDefs>();
        loaded
            .defs
            .push(make_attr("test_str", AttributeCategory::Primary, 10.0, 1.0, 30.0));
        loaded
            .defs
            .push(make_attr("test_hp", AttributeCategory::Resource, 100.0, 0.0, 999.0));
    }

    // 运行 Update 调度 → 触发 bridge 系统
    app.update();

    // 验证两条属性全部注册
    let registry = app.world_mut().resource::<AttributeRegistry>();
    assert!(registry.contains(&AttributeId::new("test_str")), "primary attr should be registered");
    assert!(registry.contains(&AttributeId::new("test_hp")), "resource attr should be registered");
    assert_eq!(registry.definitions.len(), 2);

    // 验证属性定义的数值正确
    let str_def = registry.get(&AttributeId::new("test_str")).unwrap();
    assert_eq!(str_def.default_base_value, 10.0);
    assert_eq!(str_def.min_value, 1.0);
    assert_eq!(str_def.max_value, 30.0);
    assert_eq!(str_def.category, AttributeCategory::Primary);

    let hp_def = registry.get(&AttributeId::new("test_hp")).unwrap();
    assert_eq!(hp_def.default_base_value, 100.0);
    assert_eq!(hp_def.min_value, 0.0);
    assert_eq!(hp_def.max_value, 999.0);
    assert_eq!(hp_def.category, AttributeCategory::Resource);
}

#[test]
fn bridge_skips_when_loaded_attribute_defs_empty() {
    let mut app = App::new();
    app.init_resource::<LoadedAttributeDefs>();
    app.add_plugins(AttributePlugin);

    // 不插入任何属性定义
    app.update();

    // 验证 AttributeRegistry 保持空
    let registry = app.world_mut().resource::<AttributeRegistry>();
    assert!(registry.definitions.is_empty(), "no attrs should be registered");
}

#[test]
fn bridge_rejects_attribute_with_default_out_of_range() {
    let mut app = App::new();
    app.init_resource::<LoadedAttributeDefs>();
    app.add_plugins(AttributePlugin);

    // 插入一个 default > max 的属性
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedAttributeDefs>();
        loaded
            .defs
            .push(make_attr("test_bad", AttributeCategory::Primary, 50.0, 0.0, 30.0));
    }

    app.update();

    // 验证无效属性未被注册
    let registry = app.world_mut().resource::<AttributeRegistry>();
    assert!(
        !registry.contains(&AttributeId::new("test_bad")),
        "attr with default > max should NOT be registered"
    );
    assert!(registry.definitions.is_empty(), "registry should remain empty");
}

#[test]
fn bridge_rejects_resource_with_negative_min() {
    let mut app = App::new();
    app.init_resource::<LoadedAttributeDefs>();
    app.add_plugins(AttributePlugin);

    // Resource 类别 min < 0 应被拒绝
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedAttributeDefs>();
        loaded
            .defs
            .push(make_attr("test_bad_resource", AttributeCategory::Resource, 50.0, -10.0, 100.0));
    }

    app.update();

    // 验证无效资源属性未被注册
    let registry = app.world_mut().resource::<AttributeRegistry>();
    assert!(
        !registry.contains(&AttributeId::new("test_bad_resource")),
        "resource attr with negative min should NOT be registered"
    );
}

#[test]
fn bridge_does_not_duplicate_attributes_on_second_change() {
    let mut app = App::new();
    app.init_resource::<LoadedAttributeDefs>();
    app.add_plugins(AttributePlugin);

    // 第一轮
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedAttributeDefs>();
        loaded
            .defs
            .push(make_attr("test_first", AttributeCategory::Primary, 10.0, 1.0, 30.0));
    }
    app.update();

    {
        let registry = app.world_mut().resource::<AttributeRegistry>();
        assert_eq!(registry.definitions.len(), 1);
    }

    // 第二轮（模拟热重载/增量加载）
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedAttributeDefs>();
        loaded
            .defs
            .push(make_attr("test_second", AttributeCategory::Primary, 20.0, 1.0, 30.0));
    }
    app.update();

    // 验证两个属性都存在且无重复
    let registry = app.world_mut().resource::<AttributeRegistry>();
    assert_eq!(registry.definitions.len(), 2);

    let first = registry.get(&AttributeId::new("test_first")).unwrap();
    assert_eq!(first.default_base_value, 10.0);

    let second = registry.get(&AttributeId::new("test_second")).unwrap();
    assert_eq!(second.default_base_value, 20.0);
}
