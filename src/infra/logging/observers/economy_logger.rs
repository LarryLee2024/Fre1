//! economy_logger — Economy 域日志 Observer
//!
//! 监听交易、价格、货币变更事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::economy::events::{CurrencyChanged, PriceChanged, TransactionCompleted};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 交易完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ECO001, event = "交易完成"), target = "economy")]
pub(crate) fn on_transaction_completed(trigger: On<TransactionCompleted>) {
    metrics::record(LogCode::ECO001);
    let event = trigger.event();
    info!(
        code = ?LogCode::ECO001,
        event = "交易完成",
        entity = ?event.entity,
        item = %event.item_id,
        qty = event.quantity,
        price = event.total_price,
        tx_type = ?event.transaction_type,
        "交易完成"
    );
}

/// 价格变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ECO002, event = "价格变化"), target = "economy")]
pub(crate) fn on_price_changed(trigger: On<PriceChanged>) {
    metrics::record(LogCode::ECO002);
    let event = trigger.event();
    info!(
        code = ?LogCode::ECO002,
        event = "价格变化",
        shop = %event.shop_id,
        item = %event.item_id,
        old = event.old_price,
        new = event.new_price,
        "价格变化"
    );
}

/// 货币变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::ECO003, event = "货币变化"), target = "economy")]
pub(crate) fn on_currency_changed(trigger: On<CurrencyChanged>) {
    metrics::record(LogCode::ECO003);
    let event = trigger.event();
    info!(
        code = ?LogCode::ECO003,
        event = "货币变化",
        entity = ?event.entity,
        currency = %event.currency_type,
        delta = event.delta,
        new = event.new_amount,
        reason = %event.reason,
        "货币变化"
    );
}
