//! 经济/交易领域 — 错误类型

use bevy::prelude::*;
use thiserror::Error;

/// 经济领域错误。
#[derive(Debug, Clone, Event, Error)]
pub enum EconomyError {
    /// 钱包余额不足
    #[error("insufficient funds: required={required}, available={available}")]
    InsufficientFunds { required: u64, available: u64 },
    /// 商店库存不足
    #[error("insufficient stock for '{item_id}': requested={requested}, available={available}")]
    InsufficientStock {
        item_id: String,
        requested: u32,
        available: u32,
    },
    /// 背包已满
    #[error("inventory full")]
    InventoryFull,
    /// 物品不存在
    #[error("item not found: {0}")]
    ItemNotFound(String),
    /// 商人拒收
    #[error("merchant refuses: {reason}")]
    MerchantRefuses { reason: String },
    /// 交易不符合规则
    #[error("invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
    /// 补货周期未到
    #[error("restock not ready: remaining={remaining}")]
    RestockNotReady { remaining: u32 },
}
