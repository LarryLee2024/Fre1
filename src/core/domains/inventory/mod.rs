//! inventory — 背包/物品业务领域
//!
//! SRPG 的背包与装备系统：物品管理、装备穿戴/卸下、消耗品使用、战利品生成。
//! 详见 docs/02-domain/domains/inventory_domain.md
//! 详见 docs/01-architecture/30-progression-narrative/ADR-030-progression-inventory.md

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] pub(crate) — 领域错误定义，crate 内共享
pub(crate) mod error;
// [ADR-045] pub(crate) — 业务规则失败定义，crate 内共享
pub(crate) mod failure;
// [ADR-045] pub(crate) — 领域事件定义，crate 内共享
pub(crate) mod events;
// [ADR-045] pub(crate) — 业务规则，crate 内共享
pub(crate) mod rules;
// [ADR-045] pub(crate) — ECS 系统，crate 内共享
pub(crate) mod systems;
// [ADR-045] pub(crate) — 跨域集成层，crate 内共享
pub(crate) mod integration;

#[cfg(test)]
mod tests;
