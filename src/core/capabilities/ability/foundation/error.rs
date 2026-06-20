//! 程序错误 — Ability 领域错误枚举。
//!
//! 涵盖 Ability 系统的程序错误（不应发生的异常情况）。
//! 详见 ADR-051。

use crate::core::capabilities::ability::foundation::types::AbilityInstanceId;
use crate::core::capabilities::ability::foundation::types::AbilityState;
use bevy::prelude::Event;
use thiserror::Error;

/// Ability 领域错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum AbilityError {
    /// 技能不在可激活状态（如冷却中/已移除）
    #[error("ability '{spec_id}' not ready to activate (current state: {current_state:?})")]
    NotReady {
        current_state: AbilityState,
        spec_id: String,
    },
    /// 技能有正在运行的活跃实例，不允许再次激活
    #[error("ability '{spec_id}' already has active instance {instance_id}")]
    AlreadyActive {
        spec_id: String,
        instance_id: AbilityInstanceId,
    },
    /// 技能不存在于实体的容器中
    #[error("spec '{spec_id}' not found")]
    SpecNotFound { spec_id: String },
    /// 实例不存在
    #[error("instance '{instance_id}' not found")]
    InstanceNotFound { instance_id: AbilityInstanceId },
    /// 无效的状态转换
    #[error("invalid state transition from {from:?} to {to:?}: {reason}")]
    InvalidTransition {
        from: AbilityState,
        to: AbilityState,
        reason: String,
    },
    /// Spec 未指定（激活时需要关联 Spec）
    #[error("missing spec reference for ability activation")]
    MissingSpec,
    /// 实体缺少必需的能力容器组件
    #[error("entity missing required ability container: {detail}")]
    ContainerMissing { detail: String },
}
