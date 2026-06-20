//! Attribute 领域事件
//!
//! 定义属性值生命周期中的核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/attribute_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::attribute::foundation::AttributeId;

/// 属性值发生变化时触发。
///
/// 订阅者：UI（更新属性显示）、Aggregator（级联重算依赖属性）、日志。
#[derive(Event, Debug, Clone)]
pub struct AttributeChanged {
    /// 目标实体
    pub entity: Entity,
    /// 变化的属性 ID
    pub attribute_id: AttributeId,
    /// 变化前的值
    pub old_value: f32,
    /// 变化后的值
    pub new_value: f32,
}

/// 实体首次初始化属性时触发。
///
/// 订阅者：Aggregator（注册初始属性到计算管线）、UI（初始化属性面板）。
#[derive(Event, Debug, Clone)]
pub struct AttributeInitialized {
    /// 目标实体
    pub entity: Entity,
}

/// 属性值被上限/下限截断时触发（调试/平衡用）。
///
/// 订阅者：日志、平衡分析工具。
#[derive(Event, Debug, Clone)]
pub struct AttributeClamped {
    /// 目标实体
    pub entity: Entity,
    /// 被截断的属性 ID
    pub attribute_id: AttributeId,
    /// 尝试设置的值（被截断前）
    pub attempted_value: f32,
    /// 截断后的实际值
    pub clamped_value: f32,
}
