//! ability — Ability（技能逻辑）能力领域
//!
//! 技能是角色可执行的行动模板，是技能系统的执行核心。
//! 负责技能的生命周期管理（激活→执行→完成→冷却）。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（AbilityState, AbilityInstance, CostEntry, CooldownEntry）
//! - mechanism/:  ECS 组件（ActiveAbilityContainer）+ 生命周期管理（activation/transition/cancel/cooldown）
//! - events/:     领域事件（Activated, Completed, Cancelled, CooldownStarted）
//!
//! 职责边界：
//! - 负责：技能的生命周期管理（激活→执行→完成→冷却）
//! - 不负责：技能定义模板（Spec）、执行计算（Execution）、目标选择（Targeting）
//!
//! 详见 docs/02-domain/ability_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
