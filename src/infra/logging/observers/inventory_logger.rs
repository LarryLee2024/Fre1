//! inventory_logger — Inventory 域日志 Observer
//!
//! 监听物品获取、使用、装备变更事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::inventory::events::{
    EquipmentChanged, ItemAcquired, ItemRemoved, ItemUsed, LootGenerated,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 物品获取日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV001, event = "item_acquired"))]
pub(crate) fn on_item_acquired(trigger: On<ItemAcquired>) {
    metrics::record(LogCode::INV001);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV001,
        event = "item_acquired",
        entity = ?event.entity,
        item = %event.item_template_id,
        qty = event.quantity,
        source = %event.source,
        "item_acquired"
    );
}

/// 物品使用日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV002, event = "item_used"))]
pub(crate) fn on_item_used(trigger: On<ItemUsed>) {
    metrics::record(LogCode::INV002);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV002,
        event = "item_used",
        entity = ?event.entity,
        item = %event.item_template_id,
        consumed = event.quantity_consumed,
        remaining = event.remaining,
        "item_used"
    );
}

/// 装备变更日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV003, event = "equipment_changed"))]
pub(crate) fn on_equipment_changed(trigger: On<EquipmentChanged>) {
    metrics::record(LogCode::INV003);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV003,
        event = "equipment_changed",
        entity = ?event.entity,
        slot = ?event.slot,
        "equipment_changed"
    );
}

/// 物品移除日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV004, event = "item_removed"))]
pub(crate) fn on_item_removed(trigger: On<ItemRemoved>) {
    metrics::record(LogCode::INV004);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV004,
        event = "item_removed",
        entity = ?event.entity,
        item = %event.item_template_id,
        qty = event.quantity,
        reason = ?event.reason,
        "item_removed"
    );
}

/// 战利品生成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV005, event = "loot_generated"))]
pub(crate) fn on_loot_generated(trigger: On<LootGenerated>) {
    metrics::record(LogCode::INV005);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV005,
        event = "loot_generated",
        source = ?event.source_entity,
        item_count = event.items.len(),
        "loot_generated"
    );
}
