//! Ability 领域事件
//!
//! 定义技能生命周期中的四个核心事件。
//! Bevy 0.18+ 使用 observer-based 事件系统，通过 commands.trigger() 触发。
//!
//! 详见 docs/02-domain/capabilities/ability_domain.md §6。

use bevy::prelude::*;

use crate::core::capabilities::ability::foundation::AbilityInstanceId;

/// 技能成功激活时触发。
///
/// 订阅者：Trigger（屏蔽自身触发）、UI（技能冷却/施法条显示）、日志。
#[derive(Event, Debug, Clone)]
pub struct AbilityActivated {
    /// 施法者实体
    pub entity: Entity,
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 引用的 AbilityDef ID
    pub def_id: String,
    /// 运行时实例唯一标识
    pub instance_id: AbilityInstanceId,
    /// 来源上下文描述
    pub context_desc: String,
}

/// 技能执行完毕时触发。
///
/// 订阅者：Trigger（释放频率限制位）、Progression（技能使用经验）、UI（隐藏施法条）。
#[derive(Event, Debug, Clone)]
pub struct AbilityCompleted {
    /// 施法者实体
    pub entity: Entity,
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 引用的 AbilityDef ID
    pub def_id: String,
    /// 运行时实例唯一标识
    pub instance_id: AbilityInstanceId,
    /// 执行结果描述
    pub result: String,
}

/// 技能被取消/打断时触发。
///
/// 订阅者：Trigger（释放频率限制位）、Effect（级联取消已应用的效果）、UI（取消动画）。
#[derive(Event, Debug, Clone)]
pub struct AbilityCancelled {
    /// 施法者实体
    pub entity: Entity,
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 引用的 AbilityDef ID
    pub def_id: String,
    /// 运行时实例唯一标识
    pub instance_id: AbilityInstanceId,
    /// 取消原因
    pub reason: String,
}

/// 冷却开始时触发。
///
/// 订阅者：UI（技能图标显示冷却转圈）、日志。
#[derive(Event, Debug, Clone)]
pub struct AbilityCooldownStarted {
    /// 拥有技能的实体
    pub entity: Entity,
    /// 关联的 Spec ID
    pub spec_id: String,
    /// 冷却持续回合数
    pub cooldown_duration: u32,
    /// 是否为共享冷却组
    pub shared_group: Option<String>,
}
