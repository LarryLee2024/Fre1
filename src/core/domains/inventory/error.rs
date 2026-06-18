//! 领域错误 — Inventory 域错误枚举
//!
//! 涵盖背包操作、装备、物品使用等错误。
//! 详见 docs/02-domain/domains/inventory_domain.md §4

use bevy::prelude::*;

/// 背包/物品系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum InventoryError {
    /// 背包已满，无法添加物品。
    InventoryFull { max_slots: u32 },
    /// 超过负重上限。
    ExceedsWeightLimit {
        current_weight: f32,
        max_weight: f32,
        item_weight: f32,
    },
    /// 装备条件不满足。
    EquipConditionNotMet { slot: String, reason: String },
    /// 槽位被占用（非替换操作）。
    SlotOccupied { slot: String },
    /// 背包中没有该物品。
    ItemNotFound { item_template_id: String },
    /// 物品数量不足。
    InsufficientQuantity {
        item_template_id: String,
        current: u32,
        requested: u32,
    },
    /// 物品无法使用（非消耗品）。
    ItemNotUsable { item_template_id: String },
    /// 唯一装备限制违反（is_unique 物品重复拥有）。
    UniqueItemLimit { item_template_id: String },
    /// 双手武器占用副手槽位。
    TwoHandedWeaponConflict { item_template_id: String },
}
