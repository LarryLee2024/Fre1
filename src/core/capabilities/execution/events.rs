//! Execution 领域事件
//!
//! 定义执行计算生命周期中的核心事件。
//! Bevy 0.18+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/execution_domain.md §6。

use bevy::prelude::*;

/// 执行计算完成时触发。
///
/// 订阅者：Ability（继续后续流程）、Effect（创建效果实例）、日志。
#[derive(Event, Debug, Clone)]
pub struct ExecutionCompleted {
    /// 执行类型名称
    pub execution_type: String,
    /// 来源实体
    pub source_entity: String,
    /// 目标实体
    pub target_entity: String,
    /// 计算结果数值
    pub value: f32,
    /// 是否暴击
    pub was_critical: bool,
    /// 是否未命中
    pub was_miss: bool,
    /// 关联的 AbilityDef ID
    pub ability_id: Option<String>,
}

/// 执行计算失败时触发。
///
/// 订阅者：Ability（技能执行失败处理）、日志。
#[derive(Event, Debug, Clone)]
pub struct ExecutionFailed {
    /// 执行类型名称
    pub execution_type: String,
    /// 来源实体
    pub source_entity: String,
    /// 目标实体
    pub target_entity: String,
    /// 失败原因
    pub fail_reason: String,
    /// 关联的 AbilityDef ID
    pub ability_id: Option<String>,
}

/// 自定义执行注册时触发。
///
/// 订阅者：注册中心、调试工具。
#[derive(Event, Debug, Clone)]
pub struct CustomExecutionRegistered {
    /// 自定义执行 ID
    pub execution_id: String,
    /// 描述
    pub description: String,
}
