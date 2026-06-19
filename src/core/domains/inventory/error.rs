//! 领域错误 — Inventory 域错误枚举
//!
//! 涵盖背包操作、装备、物品使用等错误。
//! 详见 docs/02-domain/domains/inventory_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

/// 背包/物品系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum InventoryError {
    /// 规则失败：背包已满，无法添加物品。
    #[error("规则失败: inventory full: max_slots={max_slots}")]
    InventoryFull { max_slots: u32 },
    /// 规则失败：超过负重上限。
    #[error(
        "规则失败: exceeds weight limit: current={current_weight}, max={max_weight}, item={item_weight}"
    )]
    ExceedsWeightLimit {
        current_weight: f32,
        max_weight: f32,
        item_weight: f32,
    },
    /// 装备条件不满足。
    #[error("equip condition not met for slot '{slot}': {reason}")]
    EquipConditionNotMet { slot: String, reason: String },
    /// 槽位被占用（非替换操作）。
    #[error("slot occupied: '{slot}'")]
    SlotOccupied { slot: String },
    /// 背包中没有该物品。
    #[error("item not found: {item_template_id}")]
    ItemNotFound { item_template_id: String },
    /// 物品数量不足。
    #[error(
        "insufficient quantity of '{item_template_id}': current={current}, requested={requested}"
    )]
    InsufficientQuantity {
        item_template_id: String,
        current: u32,
        requested: u32,
    },
    /// 物品无法使用（非消耗品）。
    #[error("item not usable: {item_template_id}")]
    ItemNotUsable { item_template_id: String },
    /// 唯一装备限制违反（is_unique 物品重复拥有）。
    #[error("unique item limit exceeded: {item_template_id}")]
    UniqueItemLimit { item_template_id: String },
    /// 双手武器占用副手槽位。
    #[error("two-handed weapon conflicts with off-hand slot: {item_template_id}")]
    TwoHandedWeaponConflict { item_template_id: String },
}
