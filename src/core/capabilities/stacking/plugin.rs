use bevy::prelude::*;

/// 堆叠能力插件。
///
/// 注册堆叠领域事件，供 Effect lifecycle 订阅消费。
pub struct StackingPlugin;

impl Plugin for StackingPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
        // 事件定义在 events.rs 中，使用 #[derive(Event)]
    }
}
