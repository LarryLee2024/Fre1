//! reaction — 反应/援护业务领域
//!
//! 管理反应槽位、触发队列、机会攻击、法术反制、护盾术、援护格挡。
//! 详见 docs/02-domain/domains/reaction_domain.md

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

#[cfg(test)]
mod tests;
