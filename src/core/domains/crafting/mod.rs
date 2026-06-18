//! crafting — 制作/锻造业务领域
//!
//! 管理配方、附魔、装备升级。
//! 详见 docs/02-domain/domains/crafting_domain.md

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
