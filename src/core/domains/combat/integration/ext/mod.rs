//! Extension traits for Bevy ECS types (EntityCommands, Query).
//!
//! These traits provide method-syntax sugar on top of Bevy's ECS primitives,
//! internally delegating to integration-layer Facade functions.
//!
//! # Design
//!
//! - Each trait follows the `*Ext` naming convention (matching `ContextExt`).
//! - Methods are facades -- they call integration facade functions, not capabilities directly.
//! - No business logic is implemented inside extension methods.
//!
//! # Usage
//!
//! ```ignore
//! use crate::core::domains::combat::integration::ext::EntityCommandsExt;
//!
//! fn my_system(mut commands: Commands) {
//!     let mut entity = commands.spawn_empty();
//!     entity.add_buff(my_buff_id);
//!     entity.heal(50);
//! }
//! ```
//!
//! See ADR-060 for full rationale.

pub mod entity_commands_ext;
pub mod query_ext;

pub use entity_commands_ext::EntityCommandsExt;
pub use query_ext::QueryExt;

#[cfg(test)]
mod tests;
