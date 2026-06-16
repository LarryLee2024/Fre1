// 触发器类型定义
// 参考：docs/01-architecture/skill-buff-abstraction.md §4.8
// 参考：docs/02-domain/trigger/trigger-rules.md

use bevy::prelude::*;

// ── Trigger 枚举 ──

/// 触发器枚举：Buff 效果触发的时机点
/// 每个变体对应回合/战斗生命周期中的具体时机
/// 参考：docs/02-domain/trigger/trigger-rules.md Trigger 定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    TurnStart,
    TurnEnd,
    BeforeAttack,
    AfterAttack,
    BeforeDamaged,
    AfterDamaged,
    BeforeMove,
    AfterMove,
    KillTarget,
    Death,
    BattleStart,
    BattleEnd,
    OnHeal,
    OnBuffApplied,
    OnBuffRemoved,
    OnRevive,
}

// ── TriggerContext ──

/// 触发上下文：Trigger 触发时携带的全部输入数据
/// 纯数据传递，不存储持久状态
#[derive(Debug, Clone)]
pub struct TriggerContext {
    pub trigger: Trigger,
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub damage_dealt: Option<i32>,
    pub is_critical: Option<bool>,
    pub chain_depth: u32,
}
