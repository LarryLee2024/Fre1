//! combat — 战斗领域
//!
//! 回合流程、先攻、伤害结算、胜负判定。
//! 详见 docs/02-domain/combat_domain.md, ADR-021
//!
//! # 模块结构
//!
//! - `plugin` — CombatPlugin（唯一对外入口）
//! - `components` — BattlePhase, TurnSubState, TurnQueue, ActionPoints
//! - `events` — 回合生命周期事件
//! - `systems` — 回合状态机各阶段 System
//! - `integration/` — 跨域访问 ACL（ADR-046）

pub(crate) mod components;
mod events;
pub(crate) mod integration;
mod plugin;
mod systems;

#[cfg(test)]
mod tests;

pub use components::*;
pub use events::*;
pub use plugin::*;
