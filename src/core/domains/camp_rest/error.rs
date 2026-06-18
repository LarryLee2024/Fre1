//! 领域错误 — CampRest 域错误枚举
//!
//! 涵盖休息请求、生命骰管理、营地事件等操作的错误。
//! 详见 docs/02-domain/domains/camp_rest_domain.md §4

use bevy::prelude::*;

/// 营地/休息系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum CampRestError {
    /// 战斗状态中无法休息。
    InCombat,
    /// 不在安全区域/非休息状态。
    NotSafe,
    /// 24 小时内已进行过长休。
    AlreadyRestedWithin24h,
    /// 长休被中断超过 1 小时。
    InterruptedTimeout { cumulative_minutes: u32 },
    /// 生命骰不足。
    InsufficientHitDice { available: u32, requested: u32 },
    /// 当前休息阶段不允许该操作。
    InvalidPhase {
        current_phase: String,
        expected: String,
    },
}
