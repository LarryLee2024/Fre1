//! 制作/锻造领域 — 资源配置

use bevy::prelude::*;
use std::collections::HashMap;

use super::components::EnchantmentDef;

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

/// 附魔定义注册表。
#[derive(Debug, Clone, Resource)]
pub struct EnchantmentDefRegistry {
    pub defs: HashMap<String, EnchantmentDef>,
}

impl EnchantmentDefRegistry {
    /// 创建空的附魔定义注册表。
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
        }
    }

    /// 注册一个附魔定义（同 ID 覆盖）。
    pub fn register(&mut self, def: EnchantmentDef) {
        self.defs.insert(def.id.clone(), def);
    }

    /// 按 ID 查询附魔定义。
    pub fn get(&self, id: &str) -> Option<&EnchantmentDef> {
        self.defs.get(id)
    }
}

impl Default for EnchantmentDefRegistry {
    fn default() -> Self {
        Self::new()
    }
}
