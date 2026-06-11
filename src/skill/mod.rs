// 技能模块：数据驱动的技能定义、槽位管理、效果预览
// 支持从 assets/skills/*.ron 外部配置文件加载

mod domain;
mod preview;
mod slots;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

// 公共 re-exports
pub use domain::*;
pub use slots::*;

/// 技能插件
pub struct SkillPlugin;

impl Plugin for SkillPlugin {
    fn build(&self, app: &mut App) {
        let registry = domain::SkillRegistry::load_from_dir("assets/skills");
        app.insert_resource(registry)
            // 注册 Reflect 类型
            .register_type::<domain::SkillTargeting>()
            .register_type::<domain::SkillCondition>()
            .register_type::<domain::SkillUseError>()
            .register_type::<domain::SkillData>()
            .register_type::<slots::SkillSlots>()
            .register_type::<slots::SkillCooldowns>();
    }
}
