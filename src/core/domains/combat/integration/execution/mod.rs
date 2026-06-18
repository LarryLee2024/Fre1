//! Execution Integration Module
//!
//! Wraps execution capability for combat damage/heal calculation.

mod facade;

pub use facade::{CombatExecutionFacade, CombatExecutionParam};

#[cfg(test)]
mod tests;
