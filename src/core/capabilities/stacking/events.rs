//! Stacking 领域事件
//!
//! 定义堆叠生命周期各阶段的事件。
//! 遵循 docs/02-domain/stacking_domain.md §6 的事件定义。
//!
//! 事件订阅关系：
//! - StackAdded / StackRemoved → Modifier（重新计算层数相关 Modifier）
//! - StackRefreshed → Effect（更新剩余持续时间）+ UI
//! - StackReplaced → Modifier（重新注册）+ UI
//! - StackOverflow → 日志 + 平衡分析

use bevy::prelude::*;

/// 堆叠层数增加时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct StackAdded {
    /// 受影响的实体 ID
    pub entity_id: String,
    /// 效果 Spec ID（效果定义 ID）
    pub effect_spec_id: String,
    /// 旧的堆叠层数
    pub old_stack: u32,
    /// 新的堆叠层数
    pub new_stack: u32,
    /// 最大堆叠层数
    pub max_stack: u32,
}

/// 堆叠层数减少时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct StackRemoved {
    /// 受影响的实体 ID
    pub entity_id: String,
    /// 效果 Spec ID（效果定义 ID）
    pub effect_spec_id: String,
    /// 旧的堆叠层数
    pub old_stack: u32,
    /// 新的堆叠层数
    pub new_stack: u32,
    /// 移除原因
    pub reason: String,
}

/// 堆叠持续时间刷新时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct StackRefreshed {
    /// 受影响的实体 ID
    pub entity_id: String,
    /// 效果 Spec ID（效果定义 ID）
    pub effect_spec_id: String,
    /// 新的持续时间
    pub new_duration: i64,
    /// 旧的持续时间
    pub old_duration: i64,
}

/// 堆叠被新实例替换时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct StackReplaced {
    /// 受影响的实体 ID
    pub entity_id: String,
    /// 效果 Spec ID（效果定义 ID）
    pub effect_spec_id: String,
    /// 旧的来源实体
    pub old_source: String,
    /// 新的来源实体
    pub new_source: String,
}

/// 堆叠达到上限触发溢出时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct StackOverflow {
    /// 受影响的实体 ID
    pub entity_id: String,
    /// 效果 Spec ID（效果定义 ID）
    pub effect_spec_id: String,
    /// 当前堆叠层数
    pub current_stack: u32,
    /// 堆叠上限
    pub limit: u32,
    /// 溢出处理动作
    pub overflow_action: String,
}
