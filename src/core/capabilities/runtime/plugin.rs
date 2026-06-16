use bevy::prelude::*;

pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register runtime pipeline, scheduler, command dispatch
    }
}
