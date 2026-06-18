//! Targeting Integration Module
//!
//! Wraps targeting capability for combat-specific target selection.

mod facade;

pub use facade::{CombatTargetingFacade, CombatTargetingParam};

#[cfg(test)]
mod tests;
