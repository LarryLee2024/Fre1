use bevy::prelude::*;

pub struct ConditionPlugin;

impl Plugin for ConditionPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.18+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
