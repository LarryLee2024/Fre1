use bevy::prelude::*;

use super::mechanism::AttributeRegistry;

pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttributeRegistry>();
    }
}
