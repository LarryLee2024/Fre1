//! Integration: 内容管线 → TagHierarchy 桥接系统
//!
//! 验证 `register_tags_from_content` 系统正确地将 `LoadedTagDefs` 中的
//! `TagDefinition` 注册到 `TagHierarchy`。
//!
//! 覆盖的场景:
//! - 正常注册: root + child 标签，验证继承掩码
//! - 空定义: 无标签定义时系统安全跳过
//! - 重复注册: 再次修改 LoadedTagDefs 不会重复注册
//! - 注册失败处理: 无效的 child（父标签不存在）被优雅拒绝

use bevy::prelude::*;

use crate::content::LoadedTagDefs;
use crate::core::capabilities::tag::foundation::{TagCategory, TagDefinition, TagId, TagNamespace};
use crate::core::capabilities::tag::mechanism::TagHierarchy;
use crate::core::capabilities::tag::plugin::TagPlugin;

/// 构造一个测试用根标签。
fn make_root(id: &str, bit_index: u32, ns: TagNamespace) -> TagDefinition {
    TagDefinition {
        id: TagId::new(id),
        path: format!("Test.{}", id),
        parent_id: None,
        bit_index,
        is_abstract: true,
        namespace: ns,
        category: TagCategory::Gameplay,
        desc_key: None,
    }
}

/// 构造一个测试用子标签。
fn make_child(id: &str, parent: &str, bit_index: u32, ns: TagNamespace) -> TagDefinition {
    TagDefinition {
        id: TagId::new(id),
        path: format!("Test.{}.{}", parent, id),
        parent_id: Some(TagId::new(parent)),
        bit_index,
        is_abstract: false,
        namespace: ns,
        category: TagCategory::Gameplay,
        desc_key: None,
    }
}

#[test]
fn bridge_registers_root_and_child_into_hierarchy() {
    let mut app = App::new();
    app.init_resource::<LoadedTagDefs>();
    app.add_plugins(TagPlugin);

    // 插入测试标签定义
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedTagDefs>();
        loaded
            .defs
            .push(make_root("test_elemental", 0, TagNamespace::Damage));
        loaded.defs.push(make_child(
            "test_fire",
            "test_elemental",
            1,
            TagNamespace::Damage,
        ));
        loaded.defs.push(make_child(
            "test_ice",
            "test_elemental",
            2,
            TagNamespace::Damage,
        ));
    }

    // 运行 Update 调度 → 触发 bridge 系统
    app.update();

    // 验证三条标签全部注册
    let hierarchy = app.world_mut().resource::<TagHierarchy>();
    assert!(
        hierarchy.tags.contains_key(&TagId::new("test_elemental")),
        "root should be registered"
    );
    assert!(
        hierarchy.tags.contains_key(&TagId::new("test_fire")),
        "child fire should be registered"
    );
    assert!(
        hierarchy.tags.contains_key(&TagId::new("test_ice")),
        "child ice should be registered"
    );

    // 验证 child parent_id 链接到 root
    let fire = hierarchy.tags.get(&TagId::new("test_fire")).unwrap();
    assert_eq!(fire.parent_id, Some(TagId::new("test_elemental")));

    // 验证继承掩码: root 包含自身和所有子标签
    let root_mask = hierarchy.inherited_mask(&TagId::new("test_elemental"));
    assert_ne!(root_mask & (1 << 0), 0, "root mask must include self");
    assert_ne!(root_mask & (1 << 1), 0, "root mask must include child fire");
    assert_ne!(root_mask & (1 << 2), 0, "root mask must include child ice");

    // 验证继承掩码: child 只包含自身
    let fire_mask = hierarchy.inherited_mask(&TagId::new("test_fire"));
    assert_ne!(fire_mask & (1 << 1), 0, "child fire mask must include self");
    assert_eq!(
        fire_mask & (1 << 0),
        0,
        "child fire mask must NOT include root"
    );
    assert_eq!(
        fire_mask & (1 << 2),
        0,
        "child fire mask must NOT include sibling ice"
    );

    // 验证 abstract_tags 集合
    assert!(
        hierarchy
            .abstract_tags
            .contains(&TagId::new("test_elemental")),
        "root should be in abstract_tags"
    );
    assert!(
        !hierarchy.abstract_tags.contains(&TagId::new("test_fire")),
        "child fire should NOT be in abstract_tags"
    );
}

#[test]
fn bridge_skips_when_loaded_tag_defs_empty() {
    let mut app = App::new();
    app.init_resource::<LoadedTagDefs>();
    app.add_plugins(TagPlugin);

    // 不插入任何标签定义，直接运行
    app.update();

    // 验证 TagHierarchy 保持空
    let hierarchy = app.world_mut().resource::<TagHierarchy>();
    assert!(hierarchy.tags.is_empty(), "no tags should be registered");
    assert!(hierarchy.children.is_empty(), "no children should exist");
    assert!(
        hierarchy.inherited_masks.is_empty(),
        "no masks should exist"
    );
}

#[test]
fn bridge_does_not_duplicate_tags_on_second_change() {
    let mut app = App::new();
    app.init_resource::<LoadedTagDefs>();
    app.add_plugins(TagPlugin);

    // 第一轮插入
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedTagDefs>();
        loaded
            .defs
            .push(make_root("test_root", 0, TagNamespace::Damage));
    }

    app.update();

    // 验证 root 已注册
    {
        let hierarchy = app.world_mut().resource::<TagHierarchy>();
        assert!(hierarchy.tags.contains_key(&TagId::new("test_root")));
        assert_eq!(hierarchy.tags.len(), 1);
    }

    // 第二轮修改 LoadedTagDefs（模拟热重载/第二轮加载）
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedTagDefs>();
        loaded
            .defs
            .push(make_root("test_second", 1, TagNamespace::Damage));
    }

    app.update();

    // 验证新标签已注册，旧标签未被影响或重复
    let hierarchy = app.world_mut().resource::<TagHierarchy>();
    assert_eq!(hierarchy.tags.len(), 2, "should have exactly 2 tags total");

    // Verify no duplicates
    let root0 = hierarchy.tags.get(&TagId::new("test_root")).unwrap();
    assert_eq!(root0.bit_index, 0, "first root bit_index unchanged");
    let root1 = hierarchy.tags.get(&TagId::new("test_second")).unwrap();
    assert_eq!(root1.bit_index, 1, "second root bit_index correct");
}

#[test]
fn bridge_handles_registration_failure_gracefully() {
    let mut app = App::new();
    app.init_resource::<LoadedTagDefs>();
    app.add_plugins(TagPlugin);

    // 插入一个 child，但其 parent 不存在
    {
        let mut loaded = app.world_mut().resource_mut::<LoadedTagDefs>();
        loaded.defs.push(make_child(
            "test_orphan",
            "test_nonexistent_parent",
            0,
            TagNamespace::Damage,
        ));
    }

    // 运行 — 系统应记录 warning 但不 panic
    app.update();

    // 验证 orphan 未被注册（因 parent 缺失）
    let hierarchy = app.world_mut().resource::<TagHierarchy>();
    assert!(
        !hierarchy.tags.contains_key(&TagId::new("test_orphan")),
        "orphan tag should NOT be registered"
    );
    assert!(hierarchy.tags.is_empty(), "hierarchy should remain empty");
}
