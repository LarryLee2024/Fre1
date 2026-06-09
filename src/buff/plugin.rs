use super::domain::BuffRegistry;
use super::resolve::resolve_status_effects;
use crate::core::registry_loader::RegistryLoader;
use crate::turn::TurnPhase;
use bevy::prelude::*;

/// Buff 插件（注册 BuffRegistry + 持续效果结算系统）
pub struct BuffPlugin;

impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        let registry = BuffRegistry::load_from_dir("assets/buffs");
        app.insert_resource(registry)
            .add_systems(OnEnter(TurnPhase::SelectUnit), resolve_status_effects);
    }
}
