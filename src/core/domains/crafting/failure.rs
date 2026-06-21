//! 规则失败 — Crafting 域业务规则不满足结果。
//!
//! 与 `CraftingError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use thiserror::Error;

/// 制作/锻造系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CraftingFailure {
    /// 材料不足。
    #[error("recipe '{recipe_id}' 材料不足: 缺少 '{missing_item}'")]
    InsufficientMaterials {
        recipe_id: String,
        missing_item: String,
    },
    /// 制作台不匹配。
    #[error("错误的 crafting station: 需要='{required}', 实际='{actual}'")]
    WrongStation { required: String, actual: String },
    /// 技能不足。
    #[error("技能不足: required_dc={required_dc}, actual_bonus={actual_bonus}")]
    InsufficientSkill { required_dc: u32, actual_bonus: i32 },
    /// 附魔槽位已满。
    #[error("enchantment 槽位已满: max={max_slots}")]
    EnchantmentSlotsFull { max_slots: u32 },
    /// 已达最大升级等级。
    #[error("已达最大升级等级: current={current}, max={max}")]
    MaxUpgradeLevel { current: u32, max: u32 },
    /// 配方未解锁。
    #[error("配方未解锁: {0}")]
    RecipeNotUnlocked(String),
    /// 互斥词条冲突。
    #[error("互斥 enchantment 冲突: {0}")]
    ExclusiveEnchantConflict(String),
}

crate::impl_rule_failure!(CraftingFailure,
    Self::InsufficientMaterials { .. } => "CRAFTING_INSUFFICIENT_MATERIALS",
    Self::WrongStation { .. } => "CRAFTING_WRONG_STATION",
    Self::InsufficientSkill { .. } => "CRAFTING_INSUFFICIENT_SKILL",
    Self::EnchantmentSlotsFull { .. } => "CRAFTING_ENCHANTMENT_SLOTS_FULL",
    Self::MaxUpgradeLevel { .. } => "CRAFTING_MAX_UPGRADE_LEVEL",
    Self::RecipeNotUnlocked { .. } => "CRAFTING_RECIPE_NOT_UNLOCKED",
    Self::ExclusiveEnchantConflict { .. } => "CRAFTING_EXCLUSIVE_ENCHANT_CONFLICT",
);
