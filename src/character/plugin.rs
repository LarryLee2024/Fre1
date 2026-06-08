use super::spawn::UnitPlugin;
use super::template::UnitTemplatePlugin;
use super::traits::TraitPlugin;
use bevy::prelude::*;

/// 角色插件（组合 Unit + Template + Trait 子插件）
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            UnitTemplatePlugin,
            TraitPlugin,
            UnitPlugin,
        ));
    }
}
