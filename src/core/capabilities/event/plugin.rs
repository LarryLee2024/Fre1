use bevy::prelude::*;

use crate::core::capabilities::event::mechanism::EventBus;

/// Event 能力插件——初始化全局事件总线。
///
/// 插入 EventBus Resource，提供全局事件路由基础设施。
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        // Resource: EventBus（全局事件路由）
        app.insert_resource(EventBus::new());
    }
}
