use super::domain::SkillRegistry;
use bevy::prelude::*;

/// 技能插件
pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        let registry = SkillRegistry::load_from_dir("assets/skills");
        app.insert_resource(registry);
    }
}
