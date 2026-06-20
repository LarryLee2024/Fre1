//! 程序错误 — Ability 领域错误枚举。
//!
//! 涵盖 Ability 系统的程序错误（不应发生的异常情况）。
//! 详见 ADR-051。

use crate::core::capabilities::ability::foundation::types::AbilityInstanceId;
use crate::core::capabilities::ability::foundation::types::AbilityState;
use thiserror::Error;

/// Ability 领域错误。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AbilityError {
    /// 技能不在可激活状态（如冷却中/已移除）
    #[error("ability '{spec_id}' not ready to activate (current state: {current_state:?})")]
    NotReady {
        current_state: AbilityState,
        spec_id: String,
    },
    /// 条件检查不通过
    #[error("condition check failed: {reason}")]
    ConditionFailed { reason: String },
    /// 资源消耗不足
    #[error("insufficient '{resource}': required {required}, available {available}")]
    InsufficientCost {
        resource: String,
        required: f32,
        available: f32,
    },
    /// 技能有正在运行的活跃实例，不允许再次激活
    #[error("ability '{spec_id}' already has active instance {instance_id}")]
    AlreadyActive {
        spec_id: String,
        instance_id: AbilityInstanceId,
    },
    /// 技能不存在于实体的容器中
    #[error("spec '{0}' not found")]
    SpecNotFound(String),
    /// 实例不存在
    #[error("instance '{0}' not found")]
    InstanceNotFound(AbilityInstanceId),
    /// 无效的状态转换
    #[error("invalid state transition from {from:?} to {to:?}: {reason}")]
    InvalidTransition {
        from: AbilityState,
        to: AbilityState,
        reason: String,
    },
    /// 冷却中不可激活
    #[error("ability '{spec_id}' on cooldown ({remaining_turns} turns remaining)")]
    OnCooldown {
        spec_id: String,
        remaining_turns: u32,
    },
    /// Spec 未指定（激活时需要关联 Spec）
    #[error("missing spec reference for ability activation")]
    MissingSpec,
    /// 通用运行时错误
    #[error("runtime error: {0}")]
    Runtime(String),
}
