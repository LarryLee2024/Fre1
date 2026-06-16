/// Ability module — 技能/能力系统的核心定义
/// 所有类型定义独立存在，skill/ 模块通过 re-export 保持向后兼容
///
/// ADR-013: SkillDef/SkillData 双类型模式、RON 配置驱动
/// ADR-014: 五阶段释放管线（Validate → Cost → Cast → Effect → Settlement）
/// ADR-015: GameplayTag 驱动的技能分类与修饰规则匹配
/// ADR-016: EffectHandler trait 扩展点
mod domain;
/// ADR-014 五阶段管线入口（prepare_skill_execution / apply_skill_costs）
pub mod pipeline;
/// 技能效果预览（伤害计算、范围展示）
mod preview;
/// SkillSlots, SkillCooldowns 槽位管理
mod slots;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

/// 公共 re-exports
pub use domain::*;
pub use slots::*;

/// Ability 插件（原 SkillPlugin）
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        let registry = domain::SkillRegistry::load_from_dir("content/skills");
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
