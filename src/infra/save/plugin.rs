use bevy::prelude::*;

use super::systems::{on_load_request, on_save_request};

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::resources::SaveManager>();
        app.init_resource::<super::resources::AutoSaveConfig>();
        app.init_resource::<super::resources::EntityRemapper>();

        app.add_observer(on_save_request);
        app.add_observer(on_load_request);

        tracing::info!("[SavePlugin] initialized (resources, observers)");
    }
}
