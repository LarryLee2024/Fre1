//! facade — Cross-cutting orchestration for localization.
//!
//! Combines storage (database + cache) for cached resolution.

pub(crate) mod resolve;

pub use resolve::resolve_cached;
