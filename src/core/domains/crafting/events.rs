//! 制作/锻造领域 — 事件定义
//!
//! 详见 docs/02-domain/domains/crafting_domain.md §6

use bevy::prelude::*;

/// 物品制作完成事件。
#[derive(Debug, Clone, Event)]
pub struct ItemCrafted {
    pub entity: Entity,
    pub recipe_id: String,
    pub output_item: String,
    pub materials_consumed: Vec<String>,
    pub skill_check_result: Option<bool>,
}

/// 附魔应用完成事件。
#[derive(Debug, Clone, Event)]
pub struct EnchantmentApplied {
    pub entity: Entity,
    pub equipment_item: String,
    pub old_enchantment: Option<String>,
    pub new_enchantment: String,
}

/// 装备升级完成事件。
#[derive(Debug, Clone, Event)]
pub struct ItemUpgraded {
    pub entity: Entity,
    pub equipment_item: String,
    pub old_level: u32,
    pub new_level: u32,
}

/// 制作失败事件。
#[derive(Debug, Clone, Event)]
pub struct CraftingFailed {
    pub entity: Entity,
    pub recipe_id: String,
    pub fail_reason: String,
    pub materials_lost: Vec<String>,
}
