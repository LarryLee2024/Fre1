//! Event Integration Module

mod facade;

pub use facade::{CombatEventFacade, CombatEventParam, CombatEventTag};

#[cfg(test)]
mod tests;
