//! 制作/锻造领域 — 资源配置

use bevy::prelude::*;

/// 制作系统配置。
#[derive(Debug, Clone, Resource)]
pub struct CraftingConfig {
    /// 制作技能检定的骰子面数（默认 d20）
    pub skill_check_die: u32,
    /// 技能检定失败时材料保留比例（0.0 = 全损，1.0 = 全保留）
    pub fail_material_retention: f32,
    /// 制作最大同时进行数
    pub max_concurrent_crafts: u32,
}

impl Default for CraftingConfig {
    fn default() -> Self {
        Self {
            skill_check_die: 20,
            fail_material_retention: 0.5,
            max_concurrent_crafts: 1,
        }
    }
}
