//! Tag 领域值对象
//!
//! 所有值对象不可变，复制时语义等价。

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::core::capabilities::tag::foundation::types::*;

/// 位掩码类型，支持最多 128 个独立标签位。
pub type BitMask = u128;

/// 标签的静态定义（运行时只读）
#[derive(Debug, Clone, Asset, Serialize, Deserialize, TypePath)]
pub struct TagDefinition {
    /// 标签唯一标识
    pub id: TagId,
    /// 层级路径名，用于人类可读的引用（如 "DamageType.Elemental.Fire"）
    pub path: String,
    /// 父标签 ID。None 表示根标签。
    pub parent_id: Option<TagId>,
    /// 分配到的位掩码索引（由 Registry 自动分配）
    pub bit_index: u32,
    /// 该标签是否为抽象标签（不可直接授予实体，仅用于层级分组）
    pub is_abstract: bool,
    /// 所属分类命名空间
    pub namespace: TagNamespace,
    /// 内容分类（Gameplay/Semantic/System），默认 Gameplay
    #[serde(default)]
    pub category: TagCategory,
    /// 描述文本本地化 Key（可选，仅在需要时提供）
    #[serde(default)]
    pub desc_key: Option<String>,
}

/// 标签查询条件定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagQuery {
    /// 匹配模式
    pub mode: TagQueryMode,
    /// 目标标签 ID 列表
    pub target_tags: Vec<TagId>,
    /// 是否考虑层级继承
    pub respect_hierarchy: bool,
}
