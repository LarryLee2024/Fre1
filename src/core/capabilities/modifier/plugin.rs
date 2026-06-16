use bevy::prelude::*;

use super::mechanism::ModifierIdGenerator;

pub struct ModifierPlugin;

impl Plugin for ModifierPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModifierIdGenerator>();
    }
}
