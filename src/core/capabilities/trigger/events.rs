//! Trigger 领域事件
//!
//! 定义触发器生命周期中的核心事件。
//! Bevy 0.18+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/trigger_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::trigger::foundation::{TriggerParams, TriggerType};

/// 触发条件满足，触发器被激活时触发。
///
/// 订阅者：Ability（创建技能实例）、日志。
#[derive(Event, Debug, Clone)]
pub struct TriggerFired {
    /// 触发源实体
    pub entity: Entity,
    /// 触发器 ID
    pub trigger_id: String,
    /// 触发类型
    pub trigger_type: TriggerType,
    /// 目标 AbilityDef ID
    pub target_ability_def_id: String,
    /// 触发上下文载荷
    pub payload: TriggerParams,
}

/// 触发器注册到实体时触发。
///
/// 订阅者：日志、调试工具。
#[derive(Event, Debug, Clone)]
pub struct TriggerRegistered {
    /// 目标实体
    pub entity: Entity,
    /// 触发器 ID
    pub trigger_id: String,
    /// 触发类型
    pub trigger_type: TriggerType,
    /// 目标 AbilityDef ID
    pub target_ability_def_id: String,
}

/// 触发器从实体移除时触发。
///
/// 订阅者：Ability（清理关联）。
#[derive(Event, Debug, Clone)]
pub struct TriggerRemoved {
    /// 目标实体
    pub entity: Entity,
    /// 触发器 ID
    pub trigger_id: String,
    /// 移除原因
    pub reason: String,
}

/// 触发器因频率限制被抑制时触发。
///
/// 订阅者：日志、平衡分析工具。
#[derive(Event, Debug, Clone)]
pub struct TriggerSuppressed {
    /// 目标实体
    pub entity: Entity,
    /// 触发器 ID
    pub trigger_id: String,
    /// 当前触发次数
    pub current_count: u32,
    /// 每回合最大触发次数
    pub max_count: u32,
}
