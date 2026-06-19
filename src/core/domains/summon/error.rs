//! 召唤领域 — 错误类型

use bevy::prelude::*;
use thiserror::Error;

/// 召唤领域错误。
#[derive(Debug, Clone, Event, Error)]
pub enum SummonError {
    /// 召唤位置不可用
    #[error("invalid summon position: {reason}")]
    InvalidPosition { reason: String },
    /// 专注冲突
    #[error("concentration conflict")]
    ConcentrationConflict,
    /// 召唤数量已达上限
    #[error("summon slot limit reached: current={current}, max={max}")]
    SlotLimitReached { current: u32, max: u32 },
    /// 模板不存在
    #[error("summon template not found: {0}")]
    TemplateNotFound(String),
    /// 嵌套召唤被禁止
    #[error("nested summon forbidden")]
    NestedSummonForbidden,
    /// 召唤者已死亡
    #[error("caster is dead")]
    CasterDead,
}
