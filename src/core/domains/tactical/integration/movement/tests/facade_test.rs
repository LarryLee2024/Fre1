use crate::core::capabilities::tag::foundation::{TagDefinition, TagId, TagNamespace};
use crate::core::capabilities::tag::mechanism::{TagHierarchy, TagSet};
use crate::core::domains::tactical::components::MovementType;
use crate::core::domains::tactical::integration::movement::facade::{
    can_move_with_type, movement_type_to_tag,
};

fn make_hierarchy_with_walk() -> TagHierarchy {
    let mut hierarchy = TagHierarchy::default();
    let tag_id = TagId::new(movement_type_to_tag(MovementType::Walk));
    let def = TagDefinition {
        id: tag_id.clone(),
        path: "MovementType.Walk".to_string(),
        parent_id: None,
        bit_index: 10,
        is_abstract: false,
        namespace: TagNamespace::Custom("Movement".to_string()),
    };
    hierarchy.tags.insert(tag_id, def);
    hierarchy
}

fn make_tagset_with_walk() -> TagSet {
    let mut ts = TagSet::empty();
    ts.bits = 1 << 10;
    ts
}

#[test]
fn can_move_walk_with_matching_tag() {
    let hierarchy = make_hierarchy_with_walk();
    let tag_set = make_tagset_with_walk();
    assert!(can_move_with_type(&tag_set, &hierarchy, MovementType::Walk));
}

#[test]
fn can_move_walk_without_tag() {
    let hierarchy = make_hierarchy_with_walk();
    let tag_set = TagSet::empty();
    assert!(!can_move_with_type(
        &tag_set,
        &hierarchy,
        MovementType::Walk
    ));
}

#[test]
fn can_move_fly_no_hierarchy_entry() {
    let hierarchy = make_hierarchy_with_walk();
    let tag_set = make_tagset_with_walk();
    assert!(!can_move_with_type(&tag_set, &hierarchy, MovementType::Fly));
}

#[test]
fn can_move_empty_hierarchy() {
    let hierarchy = TagHierarchy::default();
    let tag_set = make_tagset_with_walk();
    assert!(!can_move_with_type(
        &tag_set,
        &hierarchy,
        MovementType::Walk
    ));
}
