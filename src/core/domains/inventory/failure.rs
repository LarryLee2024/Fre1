//! 规则失败 — Inventory 域业务规则不满足结果。
//!
//! 与 `InventoryError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 docs/02-domain/domains/inventory_domain.md §4

use thiserror::Error;

/// 背包/物品系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InventoryFailure {
    /// 背包已满，无法添加物品。
    #[error("背包已满: max_slots={max_slots}")]
    Full { max_slots: u32 },
    /// 超过负重上限。
    #[error("超过负重上限: current={current_weight}, max={max_weight}, item={item_weight}")]
    ExceedsWeightLimit {
        current_weight: f32,
        max_weight: f32,
        item_weight: f32,
    },
    /// 装备条件不满足。
    #[error("槽位 '{slot}' 装备条件不满足: {reason}")]
    EquipConditionNotMet { slot: String, reason: String },
    /// 槽位被占用（非替换操作）。
    #[error("槽位被占用: '{slot}'")]
    SlotOccupied { slot: String },
    /// 物品数量不足。
    #[error("'{item_template_id}' 数量不足: current={current}, requested={requested}")]
    InsufficientQuantity {
        item_template_id: String,
        current: u32,
        requested: u32,
    },
    /// 物品无法使用（非消耗品）。
    #[error("item 不可使用: {item_template_id}")]
    ItemNotUsable { item_template_id: String },
    /// 唯一装备限制违反。
    #[error("唯一 item 数量超限: {item_template_id}")]
    UniqueItemLimit { item_template_id: String },
    /// 双手武器占用副手槽位。
    #[error("双手武器与副手槽位冲突: {item_template_id}")]
    TwoHandedWeaponConflict { item_template_id: String },
}

crate::impl_rule_failure!(InventoryFailure,
    Self::Full { .. } => "INVENTORY_FULL",
    Self::ExceedsWeightLimit { .. } => "INVENTORY_EXCEEDS_WEIGHT",
    Self::EquipConditionNotMet { .. } => "INVENTORY_EQUIP_CONDITION",
    Self::SlotOccupied { .. } => "INVENTORY_SLOT_OCCUPIED",
    Self::InsufficientQuantity { .. } => "INVENTORY_INSUFFICIENT_QTY",
    Self::ItemNotUsable { .. } => "INVENTORY_ITEM_NOT_USABLE",
    Self::UniqueItemLimit { .. } => "INVENTORY_UNIQUE_LIMIT",
    Self::TwoHandedWeaponConflict { .. } => "INVENTORY_TWO_HANDED_CONFLICT",
);
