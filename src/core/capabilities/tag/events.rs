//! 标签领域事件
//!
//! 定义标签生命周期中的核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/tag_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::tag::foundation::TagId;

/// 标签授予实体时触发。
///
/// 订阅者：Condition（触发基于标签的条件评估）、Modifier（激活标签绑定的修饰器）、UI。
#[derive(Event, Debug, Clone)]
pub struct TagAdded {
    /// 目标实体
    pub entity: Entity,
    /// 被授予的标签 ID
    pub tag_id: TagId,
    /// 标签来源
    pub source: String,
}

/// 标签从实体移除时触发。
///
/// 订阅者：Condition（触发基于标签的条件评估）、Modifier（回退标签绑定的修饰器）、UI。
#[derive(Event, Debug, Clone)]
pub struct TagRemoved {
    /// 目标实体
    pub entity: Entity,
    /// 被移除的标签 ID
    pub tag_id: TagId,
    /// 移除原因
    pub source: String,
}

/// 标签层级结构发生变更时触发（仅开发期/内容加载期）。
///
/// 订阅者：TagResolver（刷新标签继承关系缓存）。
#[derive(Event, Debug, Clone)]
pub struct TagHierarchyChanged {
    /// 发生变化的父标签
    pub parent_tag_id: TagId,
    /// 受影响的子标签列表
    pub affected_child_ids: Vec<TagId>,
}
