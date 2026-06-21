use bevy::prelude::*;

use super::load_system::{on_load_request, process_pending_load};
use super::save_system::save_world_system;

/// 存档系统 Plugin——注册 SaveManager、自动存档配置和存档 Observer。
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::resources::SaveManager>();
        app.init_resource::<super::resources::AutoSaveConfig>();
        app.init_resource::<super::resources::EntityRemapper>();

        app.add_observer(save_world_system);
        app.add_observer(on_load_request);

        app.add_systems(Update, process_pending_load);

        tracing::info!(target: "save", "[SavePlugin] 已初始化（resources, observers, systems）");
    }
}
