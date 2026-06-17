//! 标签 ECS 系统 — Observer 模式
//!
//! 响应 TagAdded/TagRemoved 事件，操作实体上的 TagSet 组件。

use bevy::prelude::*;

use crate::core::capabilities::tag::events::{TagAdded, TagRemoved};
use crate::core::capabilities::tag::mechanism::TagHierarchy;
use crate::core::capabilities::tag::mechanism::TagSet;

/// 响应 `TagAdded` 事件：将标签位添加到实体的 `TagSet`。
pub(crate) fn on_tag_added(
    trigger: On<TagAdded>,
    mut query: Query<&mut TagSet>,
    hierarchy: Res<TagHierarchy>,
) {
    let entity = trigger.entity;
    let Ok(mut tag_set) = query.get_mut(entity) else {
        return;
    };
    let Some(def) = hierarchy.tags.get(&trigger.event().tag_id) else {
        return;
    };
    tag_set.add_tag(def);
}

/// 响应 `TagRemoved` 事件：从实体的 `TagSet` 中移除标签位。
pub(crate) fn on_tag_removed(
    trigger: On<TagRemoved>,
    mut query: Query<&mut TagSet>,
    hierarchy: Res<TagHierarchy>,
) {
    let entity = trigger.entity;
    let Ok(mut tag_set) = query.get_mut(entity) else {
        return;
    };
    let Some(def) = hierarchy.tags.get(&trigger.event().tag_id) else {
        return;
    };
    tag_set.remove_tag(def);
}
