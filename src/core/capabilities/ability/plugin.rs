use bevy::prelude::*;

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
    }
}
