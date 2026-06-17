use bevy::prelude::*;

use super::mechanism::AttributeRegistry;
use super::mechanism::systems::attribute_system::on_attribute_initialized;

pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttributeRegistry>();
        app.add_observer(on_attribute_initialized);
    }
}
