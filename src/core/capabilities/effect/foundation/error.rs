//! 程序错误 — Effect 领域错误枚举。
//!
//! 涵盖 Effect 系统的程序错误（不应发生的异常情况）。
//! 详见 ADR-051。

use crate::core::capabilities::effect::foundation::types::EffectStage;
use thiserror::Error;

/// Effect 领域错误。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EffectError {
    /// 来源缺失（不变量 3.1）
    #[error("missing source: {0}")]
    MissingSource(String),
    /// 目标缺失
    #[error("missing target: {0}")]
    MissingTarget(String),
    /// 目标已有同源效果（不变量 3.5）
    #[error("duplicate effect '{def_id}': {detail}")]
    DuplicateEffect { def_id: String, detail: String },
    /// 免疫阻止（不变量 3.2）
    #[error("effect '{def_id}' blocked by immunity '{immune_tag}'")]
    ImmunityBlocked { def_id: String, immune_tag: String },
    /// 条件不满足（不变量 3.2）
    #[error("condition not met: {0}")]
    ConditionNotMet(String),
    /// 效果未找到
    #[error("effect '{0}' not found")]
    EffectNotFound(String),
    /// 周期参数非法（V6）
    #[error("invalid period: {0}")]
    InvalidPeriod(String),
    /// 阶段转换非法
    #[error("invalid transition {from:?} → {to:?}: {detail}")]
    InvalidStageTransition {
        from: EffectStage,
        to: EffectStage,
        detail: String,
    },
    /// 效果槽位已满
    #[error("effect slot limit reached ({current} / {max})")]
    SlotLimitReached { current: u32, max: u32 },
    /// 通用运行时错误
    #[error("runtime error: {0}")]
    Runtime(String),
}
