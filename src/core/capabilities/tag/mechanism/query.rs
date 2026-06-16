//! TagQuery 评估 — 纯函数，零 ECS 依赖
//!
//! 调用者负责提供 TagId → BitMask 映射：
//! - 精确匹配：映射中每个 tag_id 只包含自身位 (1 << bit_index)
//! - 层级匹配：映射中每个 tag_id 包含自身及所有子标签的位
//! `TagQuery.respect_hierarchy` 控制匹配语义而非实际位范围。

use std::collections::HashMap;

use crate::core::capabilities::tag::foundation::{BitMask, TagId, TagQuery, TagQueryMode};

/// TagId → 位掩码映射，基于 TagHierarchy 构建。
/// key = TagId, value = 该标签及其所有子标签的 BitMask
pub type InheritedMaskMap = HashMap<TagId, BitMask>;

/// 评估 TagQuery 是否匹配给定的标签位掩码。
///
/// # Arguments
/// * `query` — 查询条件
/// * `tag_bits` — 实体的当前标签位掩码
/// * `tag_masks` — TagId → BitMask 映射（调用者负责维护，确保与 hierarchy 同步）
///
/// # Returns
/// `true` 如果匹配查询条件
pub fn evaluate_query(query: &TagQuery, tag_bits: BitMask, tag_masks: &InheritedMaskMap) -> bool {
    if query.target_tags.is_empty() {
        // 空查询: Any = false, All = true, None = true
        return match query.mode {
            TagQueryMode::Any => false,
            TagQueryMode::All => true,
            TagQueryMode::None => true,
        };
    }

    match query.mode {
        TagQueryMode::Any => evaluate_any(query, tag_bits, tag_masks),
        TagQueryMode::All => evaluate_all(query, tag_bits, tag_masks),
        TagQueryMode::None => evaluate_none(query, tag_bits, tag_masks),
    }
}

fn get_mask(tag_masks: &InheritedMaskMap, tag_id: &TagId) -> BitMask {
    tag_masks.get(tag_id).copied().unwrap_or(0)
}

/// Any: tag_bits 与查询中任一目标标签的掩码有交集
fn evaluate_any(query: &TagQuery, tag_bits: BitMask, tag_masks: &InheritedMaskMap) -> bool {
    query
        .target_tags
        .iter()
        .any(|tid| (tag_bits & get_mask(tag_masks, tid)) != 0)
}

/// All: tag_bits 包含查询中所有目标标签的掩码
fn evaluate_all(query: &TagQuery, tag_bits: BitMask, tag_masks: &InheritedMaskMap) -> bool {
    query.target_tags.iter().all(|tid| {
        let mask = get_mask(tag_masks, tid);
        (tag_bits & mask) == mask
    })
}

/// None: tag_bits 与查询中任一目标标签的掩码无交集
fn evaluate_none(query: &TagQuery, tag_bits: BitMask, tag_masks: &InheritedMaskMap) -> bool {
    query
        .target_tags
        .iter()
        .all(|tid| (tag_bits & get_mask(tag_masks, tid)) == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构建 TagId → 自身位 的精确映射（非层级）
    fn exact_map(pairs: &[(&str, u32)]) -> InheritedMaskMap {
        let mut map = InheritedMaskMap::new();
        for &(id, bit) in pairs {
            map.insert(TagId::new(id), 1 << bit);
        }
        map
    }

    /// 构建 TagId → 继承位 的层级映射
    /// Fire (bit 0), Elemental (bit 1, has child Fire),
    /// Physical (bit 2, has child Slashing), Slashing (bit 3)
    fn inherited_map() -> InheritedMaskMap {
        let mut map = InheritedMaskMap::new();
        map.insert(TagId::new("tag_000001"), 1 << 0); // Fire
        map.insert(TagId::new("tag_000002"), (1 << 1) | (1 << 0)); // Elemental + Fire
        map.insert(TagId::new("tag_000003"), (1 << 2) | (1 << 3)); // Physical + Slashing
        map.insert(TagId::new("tag_000004"), 1 << 3); // Slashing
        map
    }

    #[test]
    fn unit_001_any_mode_matches_single_tag() {
        let query = TagQuery {
            mode: TagQueryMode::Any,
            target_tags: vec![TagId::new("tag_000001")], // Fire
            respect_hierarchy: false,
        };
        let map = exact_map(&[("tag_000001", 0)]);
        let bits: BitMask = 1 << 0; // has Fire
        assert!(evaluate_query(&query, bits, &map));
    }

    #[test]
    fn unit_002_any_mode_no_match() {
        let query = TagQuery {
            mode: TagQueryMode::Any,
            target_tags: vec![TagId::new("tag_000001")], // Fire
            respect_hierarchy: false,
        };
        let map = exact_map(&[("tag_000001", 0)]);
        let bits: BitMask = 0; // no tags
        assert!(!evaluate_query(&query, bits, &map));
    }

    #[test]
    fn unit_003_all_mode_matches_all() {
        let query = TagQuery {
            mode: TagQueryMode::All,
            target_tags: vec![TagId::new("tag_000001"), TagId::new("tag_000003")],
            respect_hierarchy: false,
        };
        let map = exact_map(&[("tag_000001", 0), ("tag_000003", 2)]);
        let bits: BitMask = (1 << 0) | (1 << 2); // Fire + Physical
        assert!(evaluate_query(&query, bits, &map));
    }

    #[test]
    fn unit_004_none_mode_excludes() {
        let query = TagQuery {
            mode: TagQueryMode::None,
            target_tags: vec![TagId::new("tag_000001")],
            respect_hierarchy: false,
        };
        let map = exact_map(&[("tag_000001", 0)]);
        let bits: BitMask = 1 << 2; // Physical only, no Fire
        assert!(evaluate_query(&query, bits, &map));
    }

    #[test]
    fn unit_005_hierarchy_any_matches_parent() {
        let query = TagQuery {
            mode: TagQueryMode::Any,
            target_tags: vec![TagId::new("tag_000002")], // Elemental (parent of Fire)
            respect_hierarchy: true,
        };
        let map = inherited_map();
        let bits: BitMask = 1 << 0; // has Fire (child of Elemental)
        assert!(evaluate_query(&query, bits, &map));
    }

    #[test]
    fn unit_006_empty_any_returns_false() {
        let query = TagQuery {
            mode: TagQueryMode::Any,
            target_tags: vec![],
            respect_hierarchy: false,
        };
        let map = InheritedMaskMap::new();
        assert!(!evaluate_query(&query, 0, &map));
    }

    #[test]
    fn unit_007_empty_all_returns_true() {
        let query = TagQuery {
            mode: TagQueryMode::All,
            target_tags: vec![],
            respect_hierarchy: false,
        };
        let map = InheritedMaskMap::new();
        assert!(evaluate_query(&query, 0, &map));
    }

    #[test]
    fn unit_008_hierarchy_none_excludes_child() {
        let query = TagQuery {
            mode: TagQueryMode::None,
            target_tags: vec![TagId::new("tag_000002")], // Elemental
            respect_hierarchy: true,
        };
        let map = inherited_map();
        let bits: BitMask = 0; // no tags at all
        assert!(evaluate_query(&query, bits, &map));
    }

    #[test]
    fn unit_009_all_mode_hierarchy_includes_child() {
        let query = TagQuery {
            mode: TagQueryMode::All,
            target_tags: vec![TagId::new("tag_000002")], // Elemental
            respect_hierarchy: true,
        };
        let map = inherited_map();
        let bits: BitMask = (1 << 1) | (1 << 0); // has Elemental + Fire
        assert!(evaluate_query(&query, bits, &map));
    }
}
