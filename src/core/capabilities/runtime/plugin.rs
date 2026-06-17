use bevy::prelude::*;

pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, _app: &mut App) {
        // Events（Bevy 0.18+ observer-based 事件系统）
        // Pipeline events defined in pipeline/events.rs with #[derive(Event)]
    }
}
