use bevy::prelude::*;

use crate::core::capabilities::tag::foundation::{TagDefinition, TagId, TagNamespace};
use crate::core::capabilities::tag::mechanism::lifecycle::{TagHierarchy, TagRegistrationError};

fn make_tag(
    id: &str,
    parent: Option<&str>,
    index: u32,
    ns: TagNamespace,
    abstract_: bool,
) -> TagDefinition {
    TagDefinition {
        id: TagId::new(id),
        path: String::new(),
        parent_id: parent.map(TagId::new),
        bit_index: index,
        is_abstract: abstract_,
        namespace: ns,
    }
}

#[test]
fn register_root_tag_succeeds() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut hierarchy = TagHierarchy::default();
    let def = make_tag("tag_000001", None, 0, TagNamespace::DamageType, true);
    assert!(hierarchy.register(def, &mut commands).is_ok());
    assert!(hierarchy.tags.contains_key(&TagId::new("tag_000001")));
}

#[test]
fn register_child_tag_succeeds() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut hierarchy = TagHierarchy::default();
    hierarchy
        .register(make_tag(
            "tag_000001",
            None,
            0,
            TagNamespace::DamageType,
            true,
        ), &mut commands)
        .unwrap();
    hierarchy
        .register(make_tag(
            "tag_000002",
            Some("tag_000001"),
            1,
            TagNamespace::DamageType,
            false,
        ), &mut commands)
        .unwrap();
    assert!(hierarchy.tags.contains_key(&TagId::new("tag_000001")));
    assert!(hierarchy.tags.contains_key(&TagId::new("tag_000002")));
    assert_eq!(
        hierarchy
            .children
            .get(&TagId::new("tag_000001"))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn duplicate_id_rejected() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut hierarchy = TagHierarchy::default();
    hierarchy
        .register(make_tag(
            "tag_000001",
            None,
            0,
            TagNamespace::DamageType,
            true,
        ), &mut commands)
        .unwrap();
    let result = hierarchy.register(make_tag(
        "tag_000001",
        None,
        1,
        TagNamespace::DamageType,
        true,
    ), &mut commands);
    assert!(matches!(result, Err(TagRegistrationError::DuplicateId(_))));
}

#[test]
fn parent_not_found_rejected() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut hierarchy = TagHierarchy::default();
    let result = hierarchy.register(make_tag(
        "tag_000001",
        Some("tag_999999"),
        0,
        TagNamespace::DamageType,
        false,
    ), &mut commands);
    assert!(matches!(
        result,
        Err(TagRegistrationError::ParentNotFound(_))
    ));
}

#[test]
fn circular_dependency_not_rejected() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut hierarchy = TagHierarchy::default();
    hierarchy
        .register(make_tag(
            "tag_000001",
            None,
            0,
            TagNamespace::DamageType,
            true,
        ), &mut commands)
        .unwrap();
    hierarchy
        .register(make_tag(
            "tag_000002",
            Some("tag_000001"),
            1,
            TagNamespace::DamageType,
            false,
        ), &mut commands)
        .unwrap();
    let result = hierarchy.register(make_tag(
        "tag_000003",
        Some("tag_000002"),
        2,
        TagNamespace::DamageType,
        false,
    ), &mut commands);
    assert!(result.is_ok());
    let result = hierarchy.register(make_tag(
        "tag_000004",
        Some("tag_000002"),
        3,
        TagNamespace::DamageType,
        false,
    ), &mut commands);
    assert!(result.is_ok());
}

#[test]
fn inherited_mask_contains_child_tag() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut hierarchy = TagHierarchy::default();
    hierarchy
        .register(make_tag(
            "tag_000001",
            None,
            0,
            TagNamespace::DamageType,
            true,
        ), &mut commands)
        .unwrap();
    hierarchy
        .register(make_tag(
            "tag_000002",
            Some("tag_000001"),
            1,
            TagNamespace::DamageType,
            false,
        ), &mut commands)
        .unwrap();
    let mask = hierarchy.inherited_mask(&TagId::new("tag_000001"));
    assert!(mask & (1 << 0) != 0);
    assert!(mask & (1 << 1) != 0);
}
