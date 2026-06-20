//! economy_logger — Economy 域日志 Observer
//!
//! 监听交易、价格、货币变更事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::economy::events::{CurrencyChanged, PriceChanged, TransactionCompleted};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 交易完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.economy", fields(
    code = ?LogCode::ECO001,
    event = "trade_completed",
))]
pub(crate) fn on_transaction_completed(trigger: On<TransactionCompleted>) {
    telemetry::emit(LogCode::ECO001);
    let event = trigger.event();
    info!(
        target = "domain.economy",
        entity = ?event.entity,
        item = %event.item_id,
        qty = event.quantity,
        price = event.total_price,
        tx_type = ?event.transaction_type,
        "交易完成",
    );
}

/// 价格变化日志 Observer。
#[tracing::instrument(skip_all, target = "domain.economy", fields(
    code = ?LogCode::ECO002,
    event = "shop_price_changed",
))]
pub(crate) fn on_price_changed(trigger: On<PriceChanged>) {
    telemetry::emit(LogCode::ECO002);
    let event = trigger.event();
    info!(
        target = "domain.economy",
        shop = %event.shop_id,
        item = %event.item_id,
        old = event.old_price,
        new = event.new_price,
        "价格变化",
    );
}

/// 货币变化日志 Observer。
#[tracing::instrument(skip_all, target = "domain.economy", fields(
    code = ?LogCode::ECO003,
    event = "currency_changed",
))]
pub(crate) fn on_currency_changed(trigger: On<CurrencyChanged>) {
    telemetry::emit(LogCode::ECO003);
    let event = trigger.event();
    info!(
        target = "domain.economy",
        entity = ?event.entity,
        currency = %event.currency_type,
        delta = event.delta,
        new = event.new_amount,
        reason = %event.reason,
        "货币变化",
    );
}
