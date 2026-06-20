//! 程序错误 — Effect 领域错误枚举。
//!
//! 涵盖 Effect 系统的程序错误（不应发生的异常情况）。
//! 详见 ADR-051。

use crate::core::capabilities::effect::foundation::types::EffectStage;
use bevy::prelude::Event;
use thiserror::Error;

/// Effect 领域错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum EffectError {
    /// 来源缺失（不变量 3.1）
    #[error("missing source: {detail}")]
    MissingSource { detail: String },
    /// 目标缺失
    #[error("missing target: {detail}")]
    MissingTarget { detail: String },
    /// 目标已有同源效果（不变量 3.5）
    #[error("duplicate effect '{def_id}': {detail}")]
    DuplicateEffect { def_id: String, detail: String },
    /// 免疫阻止（不变量 3.2）
    #[error("effect '{def_id}' blocked by immunity '{immune_tag}'")]
    ImmunityBlocked { def_id: String, immune_tag: String },
    /// 效果未找到
    #[error("effect '{effect_id}' not found")]
    EffectNotFound { effect_id: String },
    /// 周期参数非法（V6）
    #[error("invalid period: {reason}")]
    InvalidPeriod { reason: String },
    /// 阶段转换非法
    #[error("invalid transition {from:?} → {to:?}: {detail}")]
    InvalidStageTransition {
        from: EffectStage,
        to: EffectStage,
        detail: String,
    },
    /// 效果不可驱散（尝试 Dispelled 移除不可驱散的效果）
    #[error("effect '{instance_id}' is undispellable")]
    Undispellable { instance_id: String },
}
