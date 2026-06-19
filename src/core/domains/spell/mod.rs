//! spell — 法术业务领域
//!
//! 管理法术施放、法术位、专注、豁免。
//! 详见 docs/02-domain/domains/spell_domain.md
//! 详见 ADR-023

mod plugin;
pub use plugin::*;

// [ADR-045] private — 业务组件，仅当前模块可见
mod components;
// [ADR-045] pub(crate) — 领域错误定义，crate 内共享
pub(crate) mod error;
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
    CastingTime, MaterialComponent, SaveType, SpellComponents, SpellConfig,
    SpellDef, SpellDefId, SpellDuration, SpellLevel, SpellRange,
};

#[cfg(test)]
mod tests;
