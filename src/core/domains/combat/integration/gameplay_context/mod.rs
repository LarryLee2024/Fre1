//! GameplayContext Integration Module
//!
//! Wraps gameplay_context capability for combat action context building.

mod facade;

pub use facade::{CombatContextFacade, CombatContextParam};

#[cfg(test)]
mod tests;
