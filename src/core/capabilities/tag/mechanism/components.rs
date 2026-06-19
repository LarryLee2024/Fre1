//! ECS 组件 — 实体标签容器

use crate::core::capabilities::tag::foundation::{BitMask, TagDefinition, TagId};
use bevy::prelude::*;

/// 实体当前持有的标签集合（ECS Component）。
///
/// 使用位掩码实现 O(1) 包含检查。
/// 内部缓存了标签 ID 列表用于枚举场景，bits 变化时失效。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TagSet {
    /// 位掩码：实体当前持有的所有标签
    pub bits: BitMask,
    /// 惰性计算的标签 ID 列表缓存
    cached_tags: Vec<TagId>,
}

impl TagSet {
    /// 创建空的标签集
    pub fn empty() -> Self {
        Self {
            bits: 0,
            cached_tags: Vec::new(),
        }
    }

    /// 检查是否持有指定标签（位掩码 O(1) 检查）
    pub fn has_tag(&self, def: &TagDefinition) -> bool {
        let mask: BitMask = 1 << def.bit_index;
        (self.bits & mask) != 0
    }

    /// 检查位掩码是否包含任意一个指定的位
    pub fn has_any_bit(&self, bits: BitMask) -> bool {
        (self.bits & bits) != 0
    }

    /// 检查位掩码是否包含所有指定的位
    pub fn has_all_bits(&self, bits: BitMask) -> bool {
        (self.bits & bits) == bits
    }

    /// 添加一个标签（位操作）
    pub fn add_tag(&mut self, def: &TagDefinition) {
        let mask: BitMask = 1 << def.bit_index;
        self.bits |= mask;
        self.cached_tags.clear(); // 缓存失效
    }

    /// 移除一个标签（位操作）
    pub fn remove_tag(&mut self, def: &TagDefinition) {
        let mask: BitMask = 1 << def.bit_index;
        self.bits &= !mask;
        self.cached_tags.clear(); // 缓存失效
    }

    /// 获取所有标签 ID 列表（触发惰性计算）
    pub fn get_tags(&self) -> &[TagId] {
        &self.cached_tags
    }

    /// 从位掩码重建标签 ID 缓存
    pub fn rebuild_cache(&mut self, all_defs: &[TagDefinition]) {
        self.cached_tags = all_defs
            .iter()
            .filter(|def| {
                let mask: BitMask = 1 << def.bit_index;
                (self.bits & mask) != 0
            })
            .map(|def| def.id.clone())
            .collect();
    }
}

impl Default for TagSet {
    fn default() -> Self {
        Self::empty()
    }
}
