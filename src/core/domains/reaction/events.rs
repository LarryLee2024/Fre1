//! 领域事件 — Reaction 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! 事件订阅关系详见 docs/02-domain/domains/reaction_domain.md §6

use bevy::prelude::*;

use super::components::ReactionType;

/// 反应触发事件。
///
/// 当反应满足触发条件时发布，通知 UI/AI 进行决策。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct ReactionTriggered {
    /// 将执行反应的实体。
    pub reactor: Entity,
    /// 反应类型。
    pub reaction_type: ReactionType,
    /// 反应优先级（用于队列排序）。
    pub priority: u32,
    /// 触发的上下文描述（用于 UI 展示）。
    pub context: String,
}

/// 反应已执行事件。
///
/// 反应执行完毕后发布，通知 Combat 继续原流程。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct ReactionExecuted {
    /// 执行反应的实体。
    pub reactor: Entity,
    /// 反应类型。
    pub reaction_type: ReactionType,
    /// 执行结果摘要。
    pub result: String,
}

/// 反应被拒绝事件。
///
/// 当单位选择不使用反应时发布。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct ReactionDeclined {
    /// 拒绝反应的实体。
    pub reactor: Entity,
    /// 反应类型。
    pub reaction_type: ReactionType,
    /// 拒绝原因。
    pub reason: String,
}

/// 机会攻击已执行事件。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct OpportunityAttackExecuted {
    /// 攻击者实体。
    pub attacker: Entity,
    /// 目标实体。
    pub target: Entity,
    /// 是否命中。
    pub hit: bool,
    /// 造成的伤害（未命中则为 0）。
    pub damage: i32,
    /// 攻击是否重击。
    pub critical: bool,
}

/// 法术反制已执行事件。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct CounterspellExecuted {
    /// 反制者实体。
    pub counterer: Entity,
    /// 被反制的法术 ID。
    pub target_spell: String,
    /// 反制使用的环级。
    pub counter_level: u8,
    /// 反制是否成功。
    pub success: bool,
    /// 是否需要检定（反制环级 < 目标环级时）。
    pub check_required: bool,
    /// 检定结果（如果进行了检定）。
    pub check_roll: Option<i32>,
    /// 检定 DC（如果进行了检定）。
    pub check_dc: Option<u32>,
}

/// 护盾术已使用事件。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct ShieldUsed {
    /// 施放护盾术的实体。
    pub caster: Entity,
    /// 攻击者实体。
    pub attacker: Entity,
    /// 护盾术提供的 AC 加值。
    pub ac_bonus: i32,
    /// 护盾术生效后攻击是否仍命中。
    pub still_hit: bool,
}

/// 援护格挡已使用事件。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct GuardianUsed {
    /// 援护者实体。
    pub guardian: Entity,
    /// 被援护的目标实体。
    pub target: Entity,
    /// 转移的伤害量。
    pub transferred_damage: i32,
    /// 攻击者实体。
    pub attacker: Entity,
}
