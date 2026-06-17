use bevy::prelude::*;

pub struct TargetingPlugin;

impl Plugin for TargetingPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.18+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
