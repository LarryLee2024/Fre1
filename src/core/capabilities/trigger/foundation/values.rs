//! Trigger 值对象定义

use bevy::prelude::Reflect;

use crate::core::capabilities::trigger::foundation::types::{
    TriggerFrequency, TriggerParams, TriggerType,
};

/// 触发条件定义。
///
/// 定义触发器被激活前必须满足的额外条件。
/// 简单条件使用 `condition_id` 委托 Condition 领域；额外参数通过 `params` 传入。
#[derive(Debug, Clone, Reflect)]
pub struct TriggerCondition {
    /// 委托给 Condition 领域的条件 ID（None = 无条件）
    pub condition_id: Option<String>,
    /// 额外参数（语义由具体 TriggerType 定义）
    pub params: TriggerParams,
}

impl TriggerCondition {
    /// 创建无条件触发器条件。
    pub fn always() -> Self {
        Self {
            condition_id: None,
            params: TriggerParams::new(),
        }
    }

    /// 创建带条件引用的触发器条件。
    pub fn with_condition(condition_id: impl Into<String>) -> Self {
        Self {
            condition_id: Some(condition_id.into()),
            params: TriggerParams::new(),
        }
    }
}

/// 触发器条目——完整的触发器定义。
///
/// 定义了"什么条件下（TriggerType + TriggerCondition）激活什么技能（target_ability_def_id）"。
#[derive(Debug, Clone, Reflect)]
pub struct TriggerEntry {
    /// 触发器唯一标识
    pub id: String,
    /// 触发类型
    pub trigger_type: TriggerType,
    /// 触发条件（额外过滤条件）
    pub condition: TriggerCondition,
    /// 目标 AbilityDef ID（触发后激活的技能）
    pub target_ability_def_id: String,
    /// 触发频率控制
    pub frequency: TriggerFrequency,
    /// 是否允许在自身技能执行期间再次触发（默认 false——不变量 §4.5）
    pub allow_concurrent: bool,
}

impl TriggerEntry {
    /// 创建一个新的触发器条目。
    pub fn new(
        id: impl Into<String>,
        trigger_type: TriggerType,
        target_ability_def_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            trigger_type,
            condition: TriggerCondition::always(),
            target_ability_def_id: target_ability_def_id.into(),
            frequency: TriggerFrequency::unlimited(),
            allow_concurrent: false,
        }
    }

    /// 设置触发条件。
    pub fn with_condition(mut self, condition: TriggerCondition) -> Self {
        self.condition = condition;
        self
    }

    /// 设置频率限制。
    pub fn with_frequency(mut self, max_per_turn: u32) -> Self {
        self.frequency = TriggerFrequency::limited(max_per_turn);
        self
    }

    /// 检查当前是否允许触发（频率限制检查）。
    pub fn can_trigger(&self) -> bool {
        self.frequency.can_trigger()
    }

    /// 记录一次触发。
    pub fn record_trigger(&mut self) {
        self.frequency.record_trigger();
    }

    /// 回合结束时重置触发计数。
    pub fn reset_turn_count(&mut self) {
        self.frequency.reset_turn();
    }
}
