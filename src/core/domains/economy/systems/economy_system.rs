//! 经济/交易 Systems
//!
//! 包括购买、出售、补货等 System 和 Observer。
//! 详见 docs/02-domain/domains/economy_domain.md §5

use bevy::prelude::*;

use super::super::components::{CurrencyType, Price, Wallet};
use super::super::events::{CurrencyChanged, TransactionCompleted, TransactionType};

/// 处理购买请求。
pub fn on_purchase_request(
    _trigger: On<TransactionCompleted>,
    mut wallet_query: Query<&mut Wallet>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if event.transaction_type != TransactionType::Buy {
        return;
    }

    if let Ok(mut wallet) = wallet_query.get_mut(event.entity) {
        if !wallet.can_afford(&Price {
            base: event.total_price,
            reputation_modifier: 1.0,
            supply_modifier: 1.0,
            stolen_modifier: 1.0,
        }) {
            return;
        }

        // 简化：商店库存管理待 ShopEntity 挂接后完善
        // 当前仅处理钱包扣款

        let old_gold = wallet
            .currencies
            .get(&CurrencyType::Gold)
            .copied()
            .unwrap_or(0);
        if !wallet.deduct(&Price::new(event.total_price)) {
            return;
        }
        let new_gold = wallet
            .currencies
            .get(&CurrencyType::Gold)
            .copied()
            .unwrap_or(0);

        commands.trigger(CurrencyChanged {
            entity: event.entity,
            currency_type: "Gold".to_string(),
            old_amount: old_gold,
            new_amount: new_gold,
            delta: -(event.total_price as i64),
            reason: format!("buy:{}", event.item_id),
        });
    }
}

/// 处理出售请求。
pub fn on_sell_request(
    _trigger: On<TransactionCompleted>,
    mut wallet_query: Query<&mut Wallet>,
    mut commands: Commands,
) {
    let event = _trigger.event();
    if event.transaction_type != TransactionType::Sell {
        return;
    }

    if let Ok(mut wallet) = wallet_query.get_mut(event.entity) {
        let old_gold = wallet
            .currencies
            .get(&CurrencyType::Gold)
            .copied()
            .unwrap_or(0);
        wallet.add(CurrencyType::Gold, event.total_price);
        let new_gold = wallet
            .currencies
            .get(&CurrencyType::Gold)
            .copied()
            .unwrap_or(0);

        // 简化：商店库存更新待 ShopEntity 挂接后完善
        commands.trigger(CurrencyChanged {
            entity: event.entity,
            currency_type: "Gold".to_string(),
            old_amount: old_gold,
            new_amount: new_gold,
            delta: event.total_price as i64,
            reason: format!("sell:{}", event.item_id),
        });
    }
}
