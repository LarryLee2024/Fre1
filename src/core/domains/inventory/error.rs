//! 领域错误 — Inventory 域程序错误枚举。
//!
//! 涵盖背包系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `InventoryFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 背包/物品系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"背包已满"）请使用 [`InventoryFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum InventoryError {
    /// 背包中没有该物品。
    #[error("item not found: {item_template_id}")]
    ItemNotFound { item_template_id: String },
}
