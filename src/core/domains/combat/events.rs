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
//! | SpellCastRequested | Event | 法术施放请求 |
//! | AttackRequested | Event | 攻击请求 |

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

// ─── 命令桥接事件 ──────────────────────────────────────────────────

/// 法术施放请求（由 GameCommand handler 从 CommandExecuted 事件中桥接）。
///
/// 订阅者：Combat Execution 系统，触发法术效果执行。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SpellCastRequested {
    /// 施法者标识
    pub caster_id: String,
    /// 法术定义 ID
    pub spell_def_id: String,
    /// 目标标识
    pub target_id: String,
}

/// 攻击请求（由 GameCommand handler 从 CommandExecuted 事件中桥接）。
///
/// 订阅者：Combat Execution 系统，触发攻击效果执行。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct AttackRequested {
    /// 攻击者标识
    pub attacker_id: String,
    /// 目标标识
    pub target_id: String,
    /// 能力槽位（可选，空为普攻）
    pub ability_slot: Option<u32>,
}

// ─── 战斗动作事件 ────────────────────────────────────────────────────

/// 伤害已造成 —— 由 Combat Action System 发射。
///
/// 订阅者：HitPoints 更新系统、UI 投影、Effect 触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct DamageDealt {
    /// 目标标识
    pub target_id: String,
    /// 攻击方标识
    pub attacker_id: String,
    /// 实际伤害量
    pub damage: u32,
    /// 伤害类型（"physical", "fire", 等）
    pub damage_type: String,
    /// 是否暴击
    pub is_critical: bool,
}

/// 单位已死亡 —— 当 HitPoints 归零时由 Combat Action System 发射。
///
/// 订阅者：胜负判定系统、UI 投影、经验结算。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct UnitDied {
    /// 死亡单位标识
    pub entity_id: String,
    /// 击杀者标识
    pub killer_id: String,
}
