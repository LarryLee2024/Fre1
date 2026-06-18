//! 经济/交易领域 — 事件定义
//!
//! 详见 docs/02-domain/domains/economy_domain.md §6

use bevy::prelude::*;

/// 交易完成事件。
#[derive(Debug, Clone, Event)]
pub struct TransactionCompleted {
    pub entity: Entity,
    pub shop_id: String,
    pub item_id: String,
    pub quantity: u32,
    pub total_price: u64,
    pub price_breakdown: PriceBreakdown,
    pub transaction_type: TransactionType,
}

/// 价格明细。
#[derive(Debug, Clone)]
pub struct PriceBreakdown {
    pub base_price: u64,
    pub reputation_modifier: f32,
    pub supply_modifier: f32,
    pub stolen_modifier: f32,
    pub final_price: u64,
}

/// 交易类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Buy,
    Sell,
}

/// 价格变化事件。
#[derive(Debug, Clone, Event)]
pub struct PriceChanged {
    pub shop_id: String,
    pub item_id: String,
    pub old_price: u64,
    pub new_price: u64,
    pub reason: String,
}

/// 货币变化事件。
#[derive(Debug, Clone, Event)]
pub struct CurrencyChanged {
    pub entity: Entity,
    pub currency_type: String,
    pub old_amount: u64,
    pub new_amount: u64,
    pub delta: i64,
    pub reason: String,
}
