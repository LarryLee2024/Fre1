//! GameplayContext 领域事件
//!
//! 定义上下文生命周期中的核心事件。
//! Bevy 0.19+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/gameplay_context_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::gameplay_context::foundation::ContextOrigin;

/// 上下文构建完成时触发。
///
/// 订阅者：Execution（开始执行计算）、Ability（推进激活流程）。
#[derive(Event, Debug, Clone)]
pub struct ContextCreated {
    /// 上下文唯一标识
    pub context_id: String,
    /// 触发类型
    pub origin: ContextOrigin,
    /// 发起者实体
    pub source_entity: Entity,
    /// 目标实体
    pub target_entity: Entity,
}

/// 上下文生命周期结束时触发（消费或超时）。
///
/// 订阅者：日志、性能监控。
#[derive(Event, Debug, Clone)]
pub struct ContextConsumed {
    /// 上下文唯一标识
    pub context_id: String,
    /// 溯源链长度
    pub chain_length: u8,
    /// 上下文创建的帧号
    pub created_at_frame: u64,
}

/// 溯源链检测到循环（A→B→A）时触发（严重告警）。
///
/// 订阅者：日志、调试工具。
#[derive(Event, Debug, Clone)]
pub struct ContextCycleDetected {
    /// 上下文唯一标识
    pub context_id: String,
    /// 触发循环的节点实体
    pub cycle_node_entity: Entity,
    /// 当前链快照长度
    pub chain_length: u8,
}

/// 上下文构建校验失败时触发。
///
/// 订阅者：Ability（技能激活失败处理）、日志。
#[derive(Event, Debug, Clone)]
pub struct ContextValidationFailed {
    /// 缺失的必填字段列表
    pub missing_fields: Vec<String>,
    /// 构建时的触发类型
    pub origin: ContextOrigin,
}
