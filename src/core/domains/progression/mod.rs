//! progression — 成长养成业务领域
//!
//! SRPG 的角色成长系统：经验、等级、职业、天赋、ASI（属性值提升）。
//! 详见 docs/02-domain/domains/progression_domain.md
//! 详见 docs/01-architecture/30-progression-narrative/ADR-030-progression-inventory.md

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
pub use components::{
    ClassId, ClassLevelEntry, ClassLevels, Experience, ProgressionMarker, SubclassChoice,
    SubclassId, TalentId, TalentTree,
};
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

#[cfg(test)]
mod tests;
