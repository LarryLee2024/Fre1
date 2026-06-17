//! 条件 ECS 系统 — Observer 模式
//!
//! 响应 TagAdded/TagRemoved/AttributeChanged 事件，
//! 将 ConditionContainer 中关联的条件标记为"待重新评估"（dirty）。
//!
//! 评估是惰性的——标记 dirty 后，下次查询条件结果时会自动触发重新评估。
//! 这是 ECS 层到条件领域规则的桥梁（领域规则 §5）。
//!
//! 详见 docs/02-domain/condition_domain.md §5。

use bevy::prelude::*;

use crate::core::capabilities::attribute::events::AttributeChanged;
use crate::core::capabilities::condition::mechanism::ConditionContainer;
use crate::core::capabilities::tag::events::{TagAdded, TagRemoved};

/// 响应 `TagAdded` 事件：标记依赖该标签的条件为待重新评估。
///
/// TagId 实现了 Deref<Target=str>，可以直接传给 ConditionContainer::on_tag_changed。
pub(crate) fn on_tag_changed_by_tag_added(
    trigger: On<TagAdded>,
    mut query: Query<&mut ConditionContainer>,
) {
    let entity = trigger.entity;
    let Ok(mut container) = query.get_mut(entity) else {
        return;
    };
    container.on_tag_changed(&trigger.event().tag_id);
}

/// 响应 `TagRemoved` 事件：标记依赖该标签的条件为待重新评估。
pub(crate) fn on_tag_changed_by_tag_removed(
    trigger: On<TagRemoved>,
    mut query: Query<&mut ConditionContainer>,
) {
    let entity = trigger.entity;
    let Ok(mut container) = query.get_mut(entity) else {
        return;
    };
    container.on_tag_changed(&trigger.event().tag_id);
}

/// 响应 `AttributeChanged` 事件：标记依赖该属性的条件为待重新评估。
///
/// AttributeId 实现了 Deref<Target=str>，可以直接传给 ConditionContainer::on_attribute_changed。
pub(crate) fn on_attribute_changed(
    trigger: On<AttributeChanged>,
    mut query: Query<&mut ConditionContainer>,
) {
    let entity = trigger.entity;
    let Ok(mut container) = query.get_mut(entity) else {
        return;
    };
    container.on_attribute_changed(&trigger.event().attribute_id);
}
