//! combat — 战斗领域
//!
//! 回合流程、先攻、伤害结算、胜负判定。
//! 详见 docs/02-domain/combat_domain.md, ADR-021
//!
//! # 模块结构
//!
//! - `plugin` — CombatPlugin（唯一对外入口）
//! - `components` — BattlePhase, TurnQueue, ActionPoints
//! - `events` — 回合生命周期事件
//! - `systems` — 战斗生命周期系统 + Observer
//! - `pipeline/` — 回合流程管线（替代原 TurnSubState 状态机）
//! - `integration/` — 跨域访问 ACL（ADR-046）

pub(crate) mod components;
pub(crate) mod error;
mod events;
pub(crate) mod failure;
pub(crate) mod integration;
pub(crate) mod pipeline;
mod plugin;
mod systems;

#[cfg(test)]
mod tests;

pub use components::*;
pub use events::*;
pub use plugin::*;
