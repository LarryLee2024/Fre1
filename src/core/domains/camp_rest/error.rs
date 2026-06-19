//! 领域错误 — CampRest 域错误枚举
//!
//! 涵盖休息请求、生命骰管理、营地事件等操作的错误。
//! 详见 docs/02-domain/domains/camp_rest_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

/// 营地/休息系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CampRestError {
    /// 规则失败：战斗状态中无法休息。
    #[error("规则失败: cannot rest while in combat")]
    InCombat,
    /// 规则失败：不在安全区域/非休息状态。
    #[error("规则失败: not in a safe area")]
    NotSafe,
    /// 规则失败：24 小时内已进行过长休。
    #[error("规则失败: already rested within 24 hours")]
    AlreadyRestedWithin24h,
    /// 长休被中断超过 1 小时。
    #[error("long rest interrupted: cumulative_minutes={cumulative_minutes}")]
    InterruptedTimeout { cumulative_minutes: u32 },
    /// 生命骰不足。
    #[error("insufficient hit dice: available={available}, requested={requested}")]
    InsufficientHitDice { available: u32, requested: u32 },
    /// 当前休息阶段不允许该操作。
    #[error("invalid rest phase: current={current_phase}, expected={expected}")]
    InvalidPhase {
        current_phase: String,
        expected: String,
    },
}
