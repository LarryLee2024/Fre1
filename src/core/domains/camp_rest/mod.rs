//! camp_rest — 营地/休息业务领域
//!
//! 管理短休、长休、生命骰、营地事件。
//! 详见 docs/02-domain/domains/camp_rest_domain.md
//! 详见 ADR-031

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
