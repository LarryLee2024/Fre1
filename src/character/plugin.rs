use super::movement::animate_movement;
use super::spawn::UnitPlugin;
use super::template::UnitTemplatePlugin;
use super::traits::TraitPlugin;
use crate::turn::AppState;
use bevy::prelude::*;

/// 角色插件（组合 Unit + Template + Trait 子插件）
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((UnitTemplatePlugin, TraitPlugin, UnitPlugin))
            // 移动动画系统：只在游戏中运行
            .add_systems(Update, animate_movement.run_if(in_state(AppState::InGame)));
    }
}
