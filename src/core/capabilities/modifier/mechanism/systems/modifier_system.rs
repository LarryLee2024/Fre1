//! 修改器 ECS 系统 — Observer 模式
//!
//! 响应 ModifierApplied/ModifierRemoved 事件：
//! 将修改器插入/移出实体的 ModifierContainer。

use bevy::prelude::*;

use crate::core::capabilities::modifier::events::{ModifierApplied, ModifierRemoved};
use crate::core::capabilities::modifier::foundation::ModifierOp;
use crate::core::capabilities::modifier::mechanism::ModifierContainer;

/// 响应 `ModifierApplied` 事件：将修改器数据加入实体的 `ModifierContainer`。
pub(crate) fn on_modifier_applied(
    trigger: On<ModifierApplied>,
    mut query: Query<&mut ModifierContainer>,
) {
    let entity = trigger.entity;
    let Ok(mut container) = query.get_mut(entity) else { return; };
    let data = &trigger.event().modifier_data;

    // 按 target_attribute 分桶存储
    container
        .modifiers
        .entry(data.target_attribute.clone())
        .or_default()
        .push(data.clone());

    // Override 类型特殊处理：按实例 ID 索引
    if data.op == ModifierOp::Override {
        container
            .override_index
            .insert(data.target_attribute.clone(), data.id);
    }
}

/// 响应 `ModifierRemoved` 事件：从实体的 `ModifierContainer` 中移除修改器。
pub(crate) fn on_modifier_removed(
    trigger: On<ModifierRemoved>,
    mut query: Query<&mut ModifierContainer>,
) {
    let entity = trigger.entity;
    let Ok(mut container) = query.get_mut(entity) else { return; };
    let data = &trigger.event().modifier_data;

    // 从对应分桶中按实例 ID 移除
    if let Some(mods) = container.modifiers.get_mut(&data.target_attribute) {
        mods.retain(|m| m.id != data.id);
        if mods.is_empty() {
            container.modifiers.remove(&data.target_attribute);
        }
    }

    // 清理 override 索引
    container.override_index.remove(&data.target_attribute);
}
