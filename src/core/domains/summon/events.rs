//! 召唤领域 — 事件定义
//!
//! 详见 docs/02-domain/domains/summon_domain.md §6

use bevy::prelude::*;

/// 召唤物创建事件。
#[derive(Debug, Clone, Event)]
pub struct SummonCreated {
    pub caster: Entity,
    pub summon_entity: Entity,
    pub template_id: String,
    pub position: (i32, i32),
    pub duration_type: String,
}

/// 召唤物消失事件。
#[derive(Debug, Clone, Event)]
pub struct SummonExpired {
    pub caster: Entity,
    pub summon_entity: Entity,
    pub reason: SummonExpireReason,
}

/// 消失原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummonExpireReason {
    DurationExpired,
    ConcentrationBroken,
    CasterDied,
    Killed,
    Dismissed,
}

/// 召唤物指令事件。
#[derive(Debug, Clone, Event)]
pub struct SummonCommand {
    pub caster: Entity,
    pub summon_entity: Entity,
    pub command_type: SummonCommandType,
}

/// 指令类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummonCommandType {
    Attack(Entity),
    MoveTo(i32, i32),
    UseAbility(String),
    Follow,
    Auto,
}

/// 召唤槽位变化事件。
#[derive(Debug, Clone, Event)]
pub struct SummonSlotChanged {
    pub caster: Entity,
    pub slots_used: u32,
    pub slots_max: u32,
}
