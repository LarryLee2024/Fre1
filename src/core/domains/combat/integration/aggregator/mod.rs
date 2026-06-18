//! Aggregator Integration Module
//!
//! Wraps aggregator capability for combat attribute recalculation.

mod facade;

pub use facade::{CombatAggregatorFacade, CombatAggregatorParam};

#[cfg(test)]
mod tests;
