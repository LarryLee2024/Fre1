use crate::turn::AppState;
use bevy::prelude::*;

use super::decision::enemy_ai_system;
use super::strategy::AiStrategyRegistry;

/// AI 插件
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AiStrategyRegistry::default());
        app.add_systems(Update, enemy_ai_system.run_if(in_state(AppState::InGame)));
    }
}
