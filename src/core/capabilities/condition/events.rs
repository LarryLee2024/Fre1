//! Condition 领域事件
//!
//! 定义条件评估生命周期中的三个核心事件。
//! Bevy 0.18+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/condition_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::condition::foundation::ConditionResult;

/// 条件评估通过时触发。
///
/// 订阅者：Ability（允许继续激活）、Equipment（允许穿戴）。
#[derive(Event, Debug, Clone)]
pub struct ConditionPassed {
    /// 目标实体
    pub entity: Entity,
    /// 条件 ID
    pub condition_id: String,
    /// 评估结果数据
    pub result_data: String,
}

/// 条件评估不通过时触发。
///
/// 订阅者：Ability（阻止激活，显示失败原因）、UI（显示提示）。
#[derive(Event, Debug, Clone)]
pub struct ConditionFailed {
    /// 目标实体
    pub entity: Entity,
    /// 条件 ID
    pub condition_id: String,
    /// 失败原因
    pub fail_reason: String,
}

impl ConditionFailed {
    /// 从 ConditionResult 创建事件。
    pub fn from_result(
        entity: Entity,
        condition_id: String,
        result: &ConditionResult,
    ) -> Option<Self> {
        match result {
            ConditionResult::Passed => None,
            ConditionResult::Failed { reason } => Some(Self {
                entity,
                condition_id,
                fail_reason: reason.clone(),
            }),
        }
    }
}

/// 免疫条件生效时触发。
///
/// 订阅者：Cue（显示免疫文字弹跳）、日志。
#[derive(Event, Debug, Clone)]
pub struct ImmunityTriggered {
    /// 目标实体
    pub entity: Entity,
    /// 被免疫的效果类型
    pub effect_type: String,
    /// 生效的免疫标签
    pub immune_tag: String,
}
