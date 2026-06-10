use super::domain::{SkillData, SkillRegistry, SkillTargeting, SkillUseError};
use super::slots::{SkillCooldowns, SkillSlots};
use crate::core::registry_loader::RegistryLoader;
use crate::skill::domain::SkillCondition;
use bevy::prelude::*;

/// 技能插件
pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        let registry = SkillRegistry::load_from_dir("assets/skills");
        app.insert_resource(registry)
            // 注册 Reflect 类型
            .register_type::<SkillTargeting>()
            .register_type::<SkillCondition>()
            .register_type::<SkillUseError>()
            .register_type::<SkillData>()
            .register_type::<SkillSlots>()
            .register_type::<SkillCooldowns>();
    }
}
