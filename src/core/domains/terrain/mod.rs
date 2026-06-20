//! terrain — 地形业务领域
//!
//! SRPG 的地形基础：地形属性、表面变化、陷阱/危险区域、地形交互。
//! 详见 docs/02-domain/domains/terrain_domain.md
//! 详见 docs/04-data/domains/terrain_schema.md
//! 详见 docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] pub(crate) — 领域错误枚举，crate 内共享
pub(crate) mod error;
// [ADR-045] private — 资源，仅当前模块可见
mod resources;
// [ADR-045] pub — 领域事件定义，对外可见
pub mod events;
// [ADR-045] pub(crate) — 业务规则，crate 内共享，外部不可访问
pub(crate) mod rules;
// [ADR-045] pub(crate) — 规则失败类型，crate 内共享
pub(crate) mod failure;
// [ADR-045] pub(crate) — ECS 系统，crate 内共享
pub(crate) mod systems;

#[cfg(test)]
mod tests;
