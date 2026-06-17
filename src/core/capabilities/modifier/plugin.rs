use bevy::prelude::*;

use super::mechanism::ModifierIdGenerator;
use super::mechanism::systems::modifier_system::{on_modifier_applied, on_modifier_removed};

pub struct ModifierPlugin;

impl Plugin for ModifierPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModifierIdGenerator>();
        app.add_observer(on_modifier_applied);
        app.add_observer(on_modifier_removed);
    }
}
