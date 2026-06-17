use crate::core::capabilities::tag::foundation::{BitMask, TagId, TagQuery, TagQueryMode};
use crate::core::capabilities::tag::mechanism::query::{evaluate_query, InheritedMaskMap};

/// 构建 TagId → 自身位 的精确映射（非层级）
fn exact_map(pairs: &[(&str, u32)]) -> InheritedMaskMap {
    let mut map = InheritedMaskMap::new();
    for &(id, bit) in pairs {
        map.insert(TagId::new(id), 1 << bit);
    }
    map
}

/// 构建 TagId → 继承位 的层级映射
fn inherited_map() -> InheritedMaskMap {
    let mut map = InheritedMaskMap::new();
    map.insert(TagId::new("tag_000001"), 1 << 0);
    map.insert(TagId::new("tag_000002"), (1 << 1) | (1 << 0));
    map.insert(TagId::new("tag_000003"), (1 << 2) | (1 << 3));
    map.insert(TagId::new("tag_000004"), 1 << 3);
    map
}

#[test]
fn any_mode_matches_single_tag() {
    let query = TagQuery {
        mode: TagQueryMode::Any,
        target_tags: vec![TagId::new("tag_000001")],
        respect_hierarchy: false,
    };
    let map = exact_map(&[("tag_000001", 0)]);
    let bits: BitMask = 1 << 0;
    assert!(evaluate_query(&query, bits, &map));
}

#[test]
fn any_mode_no_match() {
    let query = TagQuery {
        mode: TagQueryMode::Any,
        target_tags: vec![TagId::new("tag_000001")],
        respect_hierarchy: false,
    };
    let map = exact_map(&[("tag_000001", 0)]);
    let bits: BitMask = 0;
    assert!(!evaluate_query(&query, bits, &map));
}

#[test]
fn all_mode_matches_all_tags() {
    let query = TagQuery {
        mode: TagQueryMode::All,
        target_tags: vec![TagId::new("tag_000001"), TagId::new("tag_000003")],
        respect_hierarchy: false,
    };
    let map = exact_map(&[("tag_000001", 0), ("tag_000003", 2)]);
    let bits: BitMask = (1 << 0) | (1 << 2);
    assert!(evaluate_query(&query, bits, &map));
}

#[test]
fn none_mode_excludes_tag() {
    let query = TagQuery {
        mode: TagQueryMode::None,
        target_tags: vec![TagId::new("tag_000001")],
        respect_hierarchy: false,
    };
    let map = exact_map(&[("tag_000001", 0)]);
    let bits: BitMask = 1 << 2;
    assert!(evaluate_query(&query, bits, &map));
}

#[test]
fn hierarchical_any_matches_parent_tag() {
    let query = TagQuery {
        mode: TagQueryMode::Any,
        target_tags: vec![TagId::new("tag_000002")],
        respect_hierarchy: true,
    };
    let map = inherited_map();
    let bits: BitMask = 1 << 0;
    assert!(evaluate_query(&query, bits, &map));
}

#[test]
fn empty_any_query_returns_false() {
    let query = TagQuery {
        mode: TagQueryMode::Any,
        target_tags: vec![],
        respect_hierarchy: false,
    };
    let map = InheritedMaskMap::new();
    assert!(!evaluate_query(&query, 0, &map));
}

#[test]
fn empty_all_query_returns_true() {
    let query = TagQuery {
        mode: TagQueryMode::All,
        target_tags: vec![],
        respect_hierarchy: false,
    };
    let map = InheritedMaskMap::new();
    assert!(evaluate_query(&query, 0, &map));
}

#[test]
fn hierarchical_none_excludes_child_tag() {
    let query = TagQuery {
        mode: TagQueryMode::None,
        target_tags: vec![TagId::new("tag_000002")],
        respect_hierarchy: true,
    };
    let map = inherited_map();
    let bits: BitMask = 0;
    assert!(evaluate_query(&query, bits, &map));
}

#[test]
fn all_mode_hierarchical_includes_child_tag() {
    let query = TagQuery {
        mode: TagQueryMode::All,
        target_tags: vec![TagId::new("tag_000002")],
        respect_hierarchy: true,
    };
    let map = inherited_map();
    let bits: BitMask = (1 << 1) | (1 << 0);
    assert!(evaluate_query(&query, bits, &map));
}
