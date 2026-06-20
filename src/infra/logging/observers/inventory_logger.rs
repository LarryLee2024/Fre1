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
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV001, event = "物品获取"), target = "inventory")]
pub(crate) fn on_item_acquired(trigger: On<ItemAcquired>) {
    metrics::record(LogCode::INV001);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV001,
        event = "物品获取",
        entity = ?event.entity,
        item = %event.item_template_id,
        qty = event.quantity,
        source = %event.source,
        "物品获取"
    );
}

/// 物品使用日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV002, event = "物品使用"), target = "inventory")]
pub(crate) fn on_item_used(trigger: On<ItemUsed>) {
    metrics::record(LogCode::INV002);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV002,
        event = "物品使用",
        entity = ?event.entity,
        item = %event.item_template_id,
        consumed = event.quantity_consumed,
        remaining = event.remaining,
        "物品使用"
    );
}

/// 装备变更日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV003, event = "装备变更"), target = "inventory")]
pub(crate) fn on_equipment_changed(trigger: On<EquipmentChanged>) {
    metrics::record(LogCode::INV003);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV003,
        event = "装备变更",
        entity = ?event.entity,
        slot = ?event.slot,
        "装备变更"
    );
}

/// 物品移除日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV004, event = "物品移除"), target = "inventory")]
pub(crate) fn on_item_removed(trigger: On<ItemRemoved>) {
    metrics::record(LogCode::INV004);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV004,
        event = "物品移除",
        entity = ?event.entity,
        item = %event.item_template_id,
        qty = event.quantity,
        reason = ?event.reason,
        "物品移除"
    );
}

/// 战利品生成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::INV005, event = "战利品生成"), target = "inventory")]
pub(crate) fn on_loot_generated(trigger: On<LootGenerated>) {
    metrics::record(LogCode::INV005);
    let event = trigger.event();
    info!(
        code = ?LogCode::INV005,
        event = "战利品生成",
        source = ?event.source_entity,
        item_count = event.items.len(),
        "战利品生成"
    );
}
