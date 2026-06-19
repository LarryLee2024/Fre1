//! 制作/锻造领域 — 错误类型

use bevy::prelude::*;
use thiserror::Error;

/// 制作领域错误。
#[derive(Debug, Clone, Event, Error)]
pub enum CraftingError {
    /// 材料不足
    #[error("insufficient materials for recipe '{recipe_id}': missing '{missing_item}'")]
    InsufficientMaterials {
        recipe_id: String,
        missing_item: String,
    },
    /// 制作台不匹配
    #[error("wrong crafting station: required='{required}', actual='{actual}'")]
    WrongStation { required: String, actual: String },
    /// 技能不足
    #[error("insufficient skill: required_dc={required_dc}, actual_bonus={actual_bonus}")]
    InsufficientSkill { required_dc: u32, actual_bonus: i32 },
    /// 附魔槽位已满
    #[error("enchantment slots full: max={max_slots}")]
    EnchantmentSlotsFull { max_slots: u32 },
    /// 已达最大升级等级
    #[error("max upgrade level reached: current={current}, max={max}")]
    MaxUpgradeLevel { current: u32, max: u32 },
    /// 配方未解锁
    #[error("recipe not unlocked: {0}")]
    RecipeNotUnlocked(String),
    /// 互斥词条冲突
    #[error("exclusive enchantment conflict: {0}")]
    ExclusiveEnchantConflict(String),
}
