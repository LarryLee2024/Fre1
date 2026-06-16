use bevy::prelude::*;

use super::mechanism::TagHierarchy;

pub struct TagPlugin;

impl Plugin for TagPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TagHierarchy>();
    }
}
