use bevy::prelude::*;

/// Trigger 能力插件——注册触发器相关的事件和 Observer。
pub struct TriggerPlugin;

impl Plugin for TriggerPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
