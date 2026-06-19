use bevy::prelude::*;

pub struct CuePlugin;

impl Plugin for CuePlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.19+ observer-based 事件系统）
        // 通过 commands.trigger() 触发，app.add_observer() 订阅
        // Events defined in events.rs with #[derive(Event)]
        // Components: CueContainerComponent available for registration
    }
}
