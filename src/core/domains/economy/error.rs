//! 经济/交易领域 — 错误类型

use bevy::prelude::*;

/// 经济领域错误。
#[derive(Debug, Clone, Event)]
pub enum EconomyError {
    /// 钱包余额不足
    InsufficientFunds { required: u64, available: u64 },
    /// 商店库存不足
    InsufficientStock {
        item_id: String,
        requested: u32,
        available: u32,
    },
    /// 背包已满
    InventoryFull,
    /// 物品不存在
    ItemNotFound(String),
    /// 商人拒收
    MerchantRefuses { reason: String },
    /// 交易不符合规则
    InvalidTransaction { reason: String },
    /// 补货周期未到
    RestockNotReady { remaining: u32 },
}
