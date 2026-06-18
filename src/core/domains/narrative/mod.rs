//! narrative — 叙事/对话业务领域
//!
//! SRPG 的叙事系统：对话树流程、故事标记、演出管理。
//! 详见 docs/02-domain/domains/narrative_domain.md

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] pub(crate) — 领域错误枚举，crate 内共享
pub(crate) mod error;
// [ADR-045] pub(crate) — 领域事件定义，crate 内共享
pub(crate) mod events;
// [ADR-045] pub(crate) — 业务规则，crate 内共享
pub(crate) mod rules;
// [ADR-045] pub(crate) — ECS 系统，crate 内共享
pub(crate) mod systems;

pub use error::*;

#[cfg(test)]
mod tests;
