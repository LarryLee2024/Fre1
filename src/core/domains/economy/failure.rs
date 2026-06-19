//! 规则失败 — Economy 域业务规则不满足结果。
//!
//! 与 `EconomyError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use thiserror::Error;

/// 经济系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EconomyFailure {
    /// 钱包余额不足。
    #[error("insufficient funds: required={required}, available={available}")]
    InsufficientFunds { required: u64, available: u64 },
    /// 商店库存不足。
    #[error("insufficient stock for '{item_id}': requested={requested}, available={available}")]
    InsufficientStock {
        item_id: String,
        requested: u32,
        available: u32,
    },
    /// 背包已满。
    #[error("inventory full")]
    InventoryFull,
    /// 商人拒收。
    #[error("merchant refuses: {reason}")]
    MerchantRefuses { reason: String },
    /// 交易不符合规则。
    #[error("invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
    /// 补货周期未到。
    #[error("restock not ready: remaining={remaining}")]
    RestockNotReady { remaining: u32 },
}

impl RuleFailure for EconomyFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::InsufficientFunds { .. } => "ECONOMY_INSUFFICIENT_FUNDS",
            Self::InsufficientStock { .. } => "ECONOMY_INSUFFICIENT_STOCK",
            Self::InventoryFull => "ECONOMY_INVENTORY_FULL",
            Self::MerchantRefuses { .. } => "ECONOMY_MERCHANT_REFUSES",
            Self::InvalidTransaction { .. } => "ECONOMY_INVALID_TRANSACTION",
            Self::RestockNotReady { .. } => "ECONOMY_RESTOCK_NOT_READY",
        }
    }
}
