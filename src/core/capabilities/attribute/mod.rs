//! attribute — 属性能力领域
//!
//! 详见 docs/02-domain/attribute_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
