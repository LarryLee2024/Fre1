use bevy::prelude::*;

/// Cue 能力插件——注册表现信号相关的事件和 Observer。
pub struct CuePlugin;

impl Plugin for CuePlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
        // 事件定义在 events.rs 中，使用 #[derive(Event)]
        // 组件：CueContainerComponent 可用于注册
    }
}
