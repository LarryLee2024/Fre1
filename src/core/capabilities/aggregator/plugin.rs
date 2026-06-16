use bevy::prelude::*;

pub struct AggregatorPlugin;

impl Plugin for AggregatorPlugin {
    fn build(&self, _app: &mut App) {
        // Events are observer-based in Bevy 0.18+:
        //   trigger via commands.trigger(...)
        //   observe via app.add_observer(...)
        // No explicit event registration needed.
    }
}
