//! 跨域共享事件 — Core 层全局事件定义
//!
//! 本文件定义多个 Domain 共享的领域事件，避免 Domain 间直接依赖。
//! 各 Domain 通过订阅这些全局事件实现跨域通信（Data Law 012）。
//!
//! 设计原则：
//! - 事件只携带最小必要信息（Entity ID + 上下文标记）
//! - 消费方 Domain 自行通过 Query 获取所需状态
//! - 新增全局事件必须更新本文件并同步文档

use bevy::prelude::*;

// ─── 回合生命周期事件 ────────────────────────────────────────────────

/// 全局回合结束事件。
///
/// 由 Combat 域在每个单位回合结束时发射，供其他 Domain 订阅。
/// 替代 Domain 间直接 import combat::OnTurnEnd 的模式。
///
/// **发射方**: combat pipeline (TurnSettlement 阶段)
/// **消费方**: terrain (表面恢复)、effect (DOT tick)、其他需要回合感知的 Domain
#[derive(Event, Debug, Clone, PartialEq)]
pub struct TurnEnded {
    /// 结束回合的单位
    pub unit: Entity,
}

/// 全局回合开始事件。
///
/// 由 Combat 域在每个单位回合开始时发射。
///
/// **发射方**: combat pipeline (Initiative 阶段)
/// **消费方**: 任何需要在回合开始时触发逻辑的 Domain
#[derive(Event, Debug, Clone, PartialEq)]
pub struct TurnStarted {
    /// 开始回合的单位
    pub unit: Entity,
}

/// 全局战斗开始事件。
///
/// **发射方**: combat battle_start_system
/// **消费方**: 任何需要在战斗开始时初始化的 Domain
#[derive(Event, Debug, Clone, PartialEq)]
pub struct BattleStarted;

/// 全局战斗结束事件。
///
/// **发射方**: combat victory_system
/// **消费方**: 任何需要在战斗结束时结算的 Domain
#[derive(Event, Debug, Clone, PartialEq)]
pub struct BattleEnded {
    /// 战斗结果：true = 胜利，false = 失败
    pub victory: bool,
}
