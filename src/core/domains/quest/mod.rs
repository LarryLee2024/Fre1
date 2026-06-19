//! quest — 任务业务领域
//!
//! 管理任务生命周期、目标追踪、奖励发放。
//! 详见 docs/02-domain/domains/quest_domain.md

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
// [ADR-045] pub(crate) — 资源定义，crate 内共享
pub(crate) mod resources;
// [ADR-045] pub(crate) — 业务规则，crate 内共享
pub(crate) mod rules;
// [ADR-045] pub(crate) — ECS 系统，crate 内共享
pub(crate) mod systems;

// ── Re-exports for external consumers (content layer + tests) ──
pub(crate) use components::{
    ObjectiveDef, ObjectiveId, ObjectiveType, QuestDef, QuestDefId, QuestRewardDef, QuestType,
};

#[cfg(test)]
mod tests;
