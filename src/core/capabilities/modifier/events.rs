//! Modifier 领域事件
//!
//! 定义修饰器生命周期中的核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/modifier_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::modifier::foundation::ModifierData;

/// 修饰器成功施加到目标时触发。
///
/// 订阅者：Aggregator（标记属性为脏 → 触发重算）、UI（更新状态面板）。
#[derive(Event, Debug, Clone)]
pub struct ModifierApplied {
    /// 受影响的实体
    pub entity: Entity,
    /// 完整的修饰器数据
    pub modifier_data: ModifierData,
}

/// 修饰器从目标移除时触发。
///
/// 订阅者：Aggregator（标记属性为脏 → 触发重算）、UI（更新状态面板）。
#[derive(Event, Debug, Clone)]
pub struct ModifierRemoved {
    /// 受影响的实体
    pub entity: Entity,
    /// 被移除的修饰器数据
    pub modifier_data: ModifierData,
}
