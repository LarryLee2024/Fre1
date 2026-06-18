//! 制作/锻造领域 — 错误类型

use bevy::prelude::*;

/// 制作领域错误。
#[derive(Debug, Clone, Event)]
pub enum CraftingError {
    /// 材料不足
    InsufficientMaterials {
        recipe_id: String,
        missing_item: String,
    },
    /// 制作台不匹配
    WrongStation { required: String, actual: String },
    /// 技能不足
    InsufficientSkill { required_dc: u32, actual_bonus: i32 },
    /// 附魔槽位已满
    EnchantmentSlotsFull { max_slots: u32 },
    /// 已达最大升级等级
    MaxUpgradeLevel { current: u32, max: u32 },
    /// 配方未解锁
    RecipeNotUnlocked(String),
    /// 互斥词条冲突
    ExclusiveEnchantConflict(String),
}
