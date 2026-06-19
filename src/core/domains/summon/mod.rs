//! summon — 召唤业务领域
//!
//! 管理召唤物创建（槽位检查/绑定）、持续时间追踪、AI 模式切换、级联消失（召唤者死亡时清理）。
//! 集成 SummonSlotManager 槽位资源，支持多召唤物上限控制。
//! 详见 docs/02-domain/domains/summon_domain.md

mod components;
mod error;
mod events;
mod failure;
mod plugin;
mod resources;
mod rules;
mod systems;

pub use components::*;
pub use error::*;
pub use events::*;
pub use plugin::*;
pub use resources::*;

#[cfg(test)]
mod tests;
