//! tactical — 战术空间业务领域
//!
//! SRPG 的物理基础：网格坐标系、单位位置、移动范围、战术判定（夹击/背刺/掩体/高地）。
//! 详见 docs/02-domain/domains/tactical_domain.md
//! 详见 docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] private — 错误类型，仅当前模块可见
mod error;
// [ADR-045] pub — 领域事件定义，对外可见
pub mod events;
// [ADR-045] pub(crate) — 域间交互入口，crate 内共享
pub(crate) mod integration;
// [ADR-045] pub(crate) — 资源，crate 内共享，外部不可访问
pub(crate) mod resources;
// [ADR-045] pub(crate) — 业务规则，crate 内共享，外部不可访问
pub(crate) mod rules;
// [ADR-045] pub(crate) — ECS 系统，crate 内共享
pub(crate) mod systems;

#[cfg(test)]
mod tests;
