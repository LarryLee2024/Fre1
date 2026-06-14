//! Buff 领域事件

use crate::shared::ids::{BuffId, UnitId};
use bevy::prelude::*;

/// Buff 已施加
#[derive(Message, Debug, Clone)]
pub struct BuffApplied {
    pub target: UnitId,
    pub target_name: String,
    pub buff_id: BuffId,
    pub source: Option<UnitId>,
    pub remaining_turns: u32,
}

/// Buff 已移除
#[derive(Message, Debug, Clone)]
pub struct BuffRemoved {
    pub target: UnitId,
    pub target_name: String,
    pub buff_id: BuffId,
    pub reason: BuffRemoveReason,
}

/// Buff 移除原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffRemoveReason {
    Expired,
    Dispelled,
    Replaced,
    Manual,
}
