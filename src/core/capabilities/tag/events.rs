//! 标签领域事件

use bevy::prelude::*;

use crate::core::capabilities::tag::foundation::TagId;

/// 标签被授予实体时触发
#[derive(Event, Debug, Clone)]
pub struct TagAdded {
    pub entity: Entity,
    pub tag_id: TagId,
    pub source: String,
}

/// 标签从实体移除时触发
#[derive(Event, Debug, Clone)]
pub struct TagRemoved {
    pub entity: Entity,
    pub tag_id: TagId,
    pub source: String,
}

/// 标签层级结构发生变更时触发（仅开发期/内容加载期）
#[derive(Event, Debug, Clone)]
pub struct TagHierarchyChanged {
    pub parent_tag_id: TagId,
    pub affected_child_ids: Vec<TagId>,
}
