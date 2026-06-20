//! inventory_logger — Inventory 域日志 Observer
//!
//! 监听物品获取、使用、装备变更事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::inventory::events::{
    EquipmentChanged, ItemAcquired, ItemRemoved, ItemUsed, LootGenerated,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 物品获取日志 Observer。
#[tracing::instrument(skip_all, target = "domain.inventory", fields(
    code = ?LogCode::INV001,
    event = "item_added",
))]
pub(crate) fn on_item_acquired(trigger: On<ItemAcquired>) {
    telemetry::emit(LogCode::INV001);
    let event = trigger.event();
    info!(
        target = "domain.inventory",
        entity = ?event.entity,
        item = %event.item_template_id,
        qty = event.quantity,
        source = %event.source,
        "物品获取",
    );
}

/// 物品使用日志 Observer。
#[tracing::instrument(skip_all, target = "domain.inventory", fields(
    code = ?LogCode::INV002,
    event = "consumable_used",
))]
pub(crate) fn on_item_used(trigger: On<ItemUsed>) {
    telemetry::emit(LogCode::INV002);
    let event = trigger.event();
    info!(
        target = "domain.inventory",
        entity = ?event.entity,
        item = %event.item_template_id,
        consumed = event.quantity_consumed,
        remaining = event.remaining,
        "物品使用",
    );
}

/// 装备变更日志 Observer。
#[tracing::instrument(skip_all, target = "domain.inventory", fields(
    code = ?LogCode::INV003,
    event = "equipment_changed",
))]
pub(crate) fn on_equipment_changed(trigger: On<EquipmentChanged>) {
    telemetry::emit(LogCode::INV003);
    let event = trigger.event();
    info!(
        target = "domain.inventory",
        entity = ?event.entity,
        slot = ?event.slot,
        "装备变更",
    );
}

/// 物品移除日志 Observer。
#[tracing::instrument(skip_all, target = "domain.inventory", fields(
    code = ?LogCode::INV004,
    event = "item_removed",
))]
pub(crate) fn on_item_removed(trigger: On<ItemRemoved>) {
    telemetry::emit(LogCode::INV004);
    let event = trigger.event();
    info!(
        target = "domain.inventory",
        entity = ?event.entity,
        item = %event.item_template_id,
        qty = event.quantity,
        reason = ?event.reason,
        "物品移除",
    );
}

/// 战利品生成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.inventory", fields(
    code = ?LogCode::INV005,
    event = "loot_generated",
))]
pub(crate) fn on_loot_generated(trigger: On<LootGenerated>) {
    telemetry::emit(LogCode::INV005);
    let event = trigger.event();
    info!(
        target = "domain.inventory",
        source = ?event.source_entity,
        item_count = event.items.len(),
        "战利品生成",
    );
}
