//! faction — 阵营关系业务领域
//!
//! SRPG 的阵营系统：阵营归属、声望管理、阵营间关系判定。
//! 详见 docs/02-domain/domains/faction_domain.md
//! 详见 docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] pub(crate) — 领域错误枚举，crate 内共享
pub(crate) mod error;
// [ADR-045] pub(crate) — 业务规则失败定义，crate 内共享
pub(crate) mod failure;
// [ADR-045] pub(crate) — 领域事件定义，crate 内共享
pub(crate) mod events;
// [ADR-045] pub(crate) — 业务规则，crate 内共享
pub(crate) mod rules;
// [ADR-045] pub(crate) — ECS 系统，crate 内共享
pub(crate) mod systems;
// [ADR-045] pub(crate) — 域集成层（Anti-Corruption Layer），crate 内共享
pub(crate) mod integration;

#[cfg(test)]
mod tests;
