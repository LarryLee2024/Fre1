//! spell — 法术业务领域
//!
//! 管理法术施放、法术位、专注、豁免。
//! 详见 docs/02-domain/domains/spell_domain.md
//! 详见 ADR-023

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
