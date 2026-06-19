//! 属性 ECS 系统 — Observer 模式
//!
//! 响应 AttributeInitialized 事件：为实体初始化属性容器中的默认值。

use bevy::prelude::*;

use crate::core::capabilities::attribute::events::{
    AttributeChanged, AttributeClamped, AttributeInitialized,
};
use crate::core::capabilities::attribute::foundation::{
    AttributeCategory, AttributeId, AttributeValue,
};
use crate::core::capabilities::attribute::mechanism::AttributeContainer;
use crate::core::capabilities::attribute::mechanism::AttributeRegistry;

/// 响应 `AttributeInitialized` 事件：根据 `AttributeRegistry` 中的定义，
/// 为实体的 `AttributeContainer` 填充默认属性值。
pub(crate) fn on_attribute_initialized(
    trigger: On<AttributeInitialized>,
    mut query: Query<&mut AttributeContainer>,
    registry: Res<AttributeRegistry>,
) {
    let entity = trigger.entity;
    let Ok(mut container) = query.get_mut(entity) else {
        return;
    };

    for def in registry.definitions.values() {
        // 仅初始化尚未存在的属性（不覆盖已有值）
        if !container.attributes.contains_key(&def.id) {
            container.attributes.insert(
                def.id.clone(),
                AttributeValue {
                    def_id: def.id.clone(),
                    base_value: def.default_base_value,
                    current_value: def.default_base_value,
                    aggregator_managed: def.category == AttributeCategory::Derived,
                },
            );
        }
    }
}

/// 为实体初始化属性并触发 AttributeInitialized 事件。
///
/// 外部调用方在确保实体拥有 AttributeContainer 组件后调用此函数。
pub(crate) fn initialize_attributes(
    mut commands: Commands,
    entity: Entity,
) {
    commands.trigger(AttributeInitialized { entity });
}

/// 触发 AttributeChanged 事件（属性值变化后调用）。
pub(crate) fn notify_attribute_changed(
    mut commands: Commands,
    entity: Entity,
    attribute_id: AttributeId,
    old_value: f32,
    new_value: f32,
) {
    commands.trigger(AttributeChanged {
        entity,
        attribute_id,
        old_value,
        new_value,
    });
}

/// 触发 AttributeClamped 事件（属性值被截断时调用）。
pub(crate) fn notify_attribute_clamped(
    mut commands: Commands,
    entity: Entity,
    attribute_id: AttributeId,
    attempted_value: f32,
    clamped_value: f32,
) {
    commands.trigger(AttributeClamped {
        entity,
        attribute_id,
        attempted_value,
        clamped_value,
    });
}
