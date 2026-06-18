//! summon — 召唤业务领域
//!
//! 管理召唤物创建、绑定、持续时间、消失。
//! 详见 docs/02-domain/domains/summon_domain.md

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
