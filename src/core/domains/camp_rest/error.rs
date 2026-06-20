//! 领域错误 — CampRest 域程序错误枚举。
//!
//! 涵盖营地/休息系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `CampRestFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 营地/休息系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"战斗中无法休息"）请使用 [`CampRestFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CampRestError {
    /// 长休被中断超过 1 小时。
    #[error("long rest interrupted: cumulative_minutes={cumulative_minutes}")]
    InterruptedTimeout { cumulative_minutes: u32 },
    /// 当前休息阶段不允许该操作。
    #[error("invalid rest phase: current={current_phase}, expected={expected}")]
    InvalidPhase {
        current_phase: String,
        expected: String,
    },
}
