//! 领域事件 — Combat 域对外发布的事件
//!
//! 所有跨域通信必须通过 Event，禁止直接引用对方数据结构（Data Law 012）。
//!
//! # 事件分类
//!
//! | 事件 | 机制 | 用途 |
//! |------|------|------|
//! | OnTurnStart | Trigger | "回合开始时"类效果 |
//! | OnTurnEnd | Trigger | "回合结束时"类效果（Buff Tick, DOT） |
//! | UnitActionComplete | Event | 外部通知"我行动完了" |
//! | BetweenTurns | Trigger | 队伍切换时 |
//! | OnRoundEnd | Trigger | 所有队伍结束一轮 |
//! | OnBattleStart | Trigger | 战斗开始初始化 |
//! | OnBattleEnd | Trigger | 战斗结束结算 |

use bevy::prelude::*;

use super::components::TeamId;

// ─── 回合生命周期事件 ────────────────────────────────────────────────

/// 单位回合开始时触发。
///
/// Trigger 事件，通过 `commands.trigger(OnTurnStart { unit })` 发射。
/// 订阅者：Buff（"每回合开始时"）、被动技能、UI 高亮。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct OnTurnStart {
    /// 当前回合的单位
    pub unit: Entity,
}

/// 单位回合结束时触发。
///
/// Trigger 事件，在 TurnSettlement 阶段发射。
/// 订阅者：Buff（"每回合结束时"）、DOT、冷却推进。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct OnTurnEnd {
    /// 当前回合的单位
    pub unit: Entity,
}

/// 外部通知回合系统"行动已完成"。
///
/// 普通 Event，由 UnitAction 阶段的 System 消费。
/// 战斗系统（Ability/Execution）在行动完成后发射此事件。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct UnitActionComplete {
    /// 完成行动的单位
    pub unit: Entity,
}

// ─── 队伍/轮次事件 ──────────────────────────────────────────────────

/// 切换到新队伍时触发。
///
/// TurnEnd 阶段检测到队伍切换时发射。
/// 订阅者：领域效果、环境效果、UI 更新。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct BetweenTurns {
    /// 当前行动队伍
    pub team: TeamId,
}

/// 所有队伍结束一轮（一轮循环完成）时触发。
///
/// Trigger 事件。
/// 订阅者：全局结算、召唤物消失、全局效果 Tick。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct OnRoundEnd {
    /// 已完成的轮数
    pub round: u32,
}

// ─── 战斗生命周期事件 ────────────────────────────────────────────────

/// 战斗开始时触发（BattlePhase 进入 Battle 时）。
///
/// Trigger 事件。
/// 订阅者：初始化战斗相关系统、UI 切换到战斗界面。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct OnBattleStart;

/// 战斗结束结果。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum BattleResult {
    Victory,
    Defeat,
}

/// 战斗结束时触发。
///
/// Trigger 事件。
/// 订阅者：经验结算、战利品、回放结束。
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
pub struct OnBattleEnd {
    /// 战斗结果
    pub result: BattleResult,
}
