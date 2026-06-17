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
