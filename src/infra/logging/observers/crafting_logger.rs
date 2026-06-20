//! crafting_logger — Crafting 域日志 Observer
//!
//! 监听制作、附魔、升级事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::crafting::events::{
    CraftingFailed, EnchantmentApplied, ItemCrafted, ItemUpgraded,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 制作完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.crafting", fields(
    code = ?LogCode::CRF003,
    event = "crafting_completed",
))]
pub(crate) fn on_item_crafted(trigger: On<ItemCrafted>) {
    telemetry::emit(LogCode::CRF003);
    let event = trigger.event();
    info!(
        target = "domain.crafting",
        entity = ?event.entity,
        recipe = %event.recipe_id,
        output = %event.output_item,
        "制作完成",
    );
}

/// 附魔应用日志 Observer。
#[tracing::instrument(skip_all, target = "domain.crafting", fields(
    code = ?LogCode::CRF003,
    event = "enchantment_applied",
))]
pub(crate) fn on_enchantment_applied(trigger: On<EnchantmentApplied>) {
    telemetry::emit(LogCode::CRF003);
    let event = trigger.event();
    info!(
        target = "domain.crafting",
        entity = ?event.entity,
        equipment = %event.equipment_item,
        enchantment = %event.new_enchantment,
        "附魔应用",
    );
}

/// 装备升级日志 Observer。
#[tracing::instrument(skip_all, target = "domain.crafting", fields(
    code = ?LogCode::CRF003,
    event = "item_upgraded",
))]
pub(crate) fn on_item_upgraded(trigger: On<ItemUpgraded>) {
    telemetry::emit(LogCode::CRF003);
    let event = trigger.event();
    info!(
        target = "domain.crafting",
        entity = ?event.entity,
        equipment = %event.equipment_item,
        old = event.old_level,
        new = event.new_level,
        "装备升级",
    );
}

/// 制作失败日志 Observer。
#[tracing::instrument(skip_all, target = "domain.crafting", fields(
    code = ?LogCode::CRF004,
    event = "crafting_failed",
))]
pub(crate) fn on_crafting_failed(trigger: On<CraftingFailed>) {
    telemetry::emit(LogCode::CRF004);
    let event = trigger.event();
    warn!(
        target = "domain.crafting",
        entity = ?event.entity,
        recipe = %event.recipe_id,
        reason = %event.fail_reason,
        "制作失败",
    );
}
