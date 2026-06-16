use bevy::prelude::*;

pub struct GameplayContextPlugin;

impl Plugin for GameplayContextPlugin {
    fn build(&self, _app: &mut App) {
        // GameplayContext is a pure data carrier with no ECS resources or systems.
        // Events use Bevy 0.18 observer pattern — derived with #[derive(Event)];
        // no explicit registration needed.
    }
}
