//! crafting_logger — Crafting 域日志 Observer
//!
//! 监听制作、附魔、升级事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::crafting::events::{
    CraftingFailed, EnchantmentApplied, ItemCrafted, ItemUpgraded,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 制作完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CRF003, event = "制作完成"), target = "crafting")]
pub(crate) fn on_item_crafted(trigger: On<ItemCrafted>) {
    metrics::record(LogCode::CRF003);
    let event = trigger.event();
    info!(
        code = ?LogCode::CRF003,
        event = "制作完成",
        entity = ?event.entity,
        recipe = %event.recipe_id,
        output = %event.output_item,
        "制作完成"
    );
}

/// 附魔应用日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CRF003, event = "附魔应用"), target = "crafting")]
pub(crate) fn on_enchantment_applied(trigger: On<EnchantmentApplied>) {
    metrics::record(LogCode::CRF003);
    let event = trigger.event();
    info!(
        code = ?LogCode::CRF003,
        event = "附魔应用",
        entity = ?event.entity,
        equipment = %event.equipment_item,
        enchantment = %event.new_enchantment,
        "附魔应用"
    );
}

/// 装备升级日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CRF003, event = "装备升级"), target = "crafting")]
pub(crate) fn on_item_upgraded(trigger: On<ItemUpgraded>) {
    metrics::record(LogCode::CRF003);
    let event = trigger.event();
    info!(
        code = ?LogCode::CRF003,
        event = "装备升级",
        entity = ?event.entity,
        equipment = %event.equipment_item,
        old = event.old_level,
        new = event.new_level,
        "装备升级"
    );
}

/// 制作失败日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::CRF004, event = "制作失败"), target = "crafting")]
pub(crate) fn on_crafting_failed(trigger: On<CraftingFailed>) {
    metrics::record(LogCode::CRF004);
    let event = trigger.event();
    warn!(
        code = ?LogCode::CRF004,
        event = "制作失败",
        entity = ?event.entity,
        recipe = %event.recipe_id,
        reason = %event.fail_reason,
        "制作失败"
    );
}
