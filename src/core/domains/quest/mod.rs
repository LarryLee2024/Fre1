//! quest — 任务业务领域
//!
//! 管理任务生命周期、目标追踪、奖励发放。
//! 详见 docs/02-domain/domains/quest_domain.md

mod components;
mod error;
mod events;
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
