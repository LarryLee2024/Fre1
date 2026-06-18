//! economy — 经济/交易业务领域
//!
//! 管理货币、商店、交易流程。
//! 详见 docs/02-domain/domains/economy_domain.md

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
