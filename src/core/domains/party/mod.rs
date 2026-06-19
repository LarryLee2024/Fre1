//! party — 队伍业务领域
//!
//! 管理角色编成、阵型配置、羁绊系统。
//! 详见 docs/02-domain/domains/party_domain.md
//! 详见 ADR-031

mod components;
mod error;
pub(crate) mod events;
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
