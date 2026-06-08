use super::domain::BuffRegistry;
use bevy::prelude::*;

/// Buff 插件
pub struct BuffPlugin;

impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        let registry = BuffRegistry::load_from_dir("assets/buffs");
        app.insert_resource(registry);
    }
}
