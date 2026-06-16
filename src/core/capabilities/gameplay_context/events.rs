//! GameplayContext 领域事件

use bevy::prelude::*;

use crate::core::capabilities::gameplay_context::foundation::ContextOrigin;

/// 上下文构建完成事件。
#[derive(Event, Debug, Clone)]
pub struct ContextCreated {
    /// 上下文 ID
    pub context_id: String,
    /// 触发类型
    pub origin: ContextOrigin,
    /// 发起者实体
    pub source_entity: Entity,
    /// 目标实体
    pub target_entity: Entity,
}

/// 上下文生命周期结束事件。
#[derive(Event, Debug, Clone)]
pub struct ContextConsumed {
    /// 上下文 ID
    pub context_id: String,
    /// 溯源链长度
    pub chain_length: u8,
    /// 创建帧号
    pub created_at_frame: u64,
}

/// 溯源链检测到循环事件。
#[derive(Event, Debug, Clone)]
pub struct ContextCycleDetected {
    /// 上下文 ID
    pub context_id: String,
    /// 触发循环的节点
    pub cycle_node_entity: Entity,
    /// 当前链快照长度
    pub chain_length: u8,
}

/// 上下文构建校验失败事件。
#[derive(Event, Debug, Clone)]
pub struct ContextValidationFailed {
    /// 缺失字段列表
    pub missing_fields: Vec<String>,
    /// 触发类型（构建时的 origin）
    pub origin: ContextOrigin,
}
