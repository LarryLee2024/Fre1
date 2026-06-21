//! Module Name: Projections — Domain Event to ViewModel projection pipeline
//!
//! Pure functions that transform domain events into ViewModel updates.
//! Each projection module handles one domain area (battle, character, etc.).
//! Projections are stateless, deterministic, and independently testable.
//!
//! See `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

pub mod battle;

pub use battle::*;
