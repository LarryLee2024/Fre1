//! Event Integration Module

mod facade;

pub use crate::core::capabilities::event::mechanism::EventBus;
pub use facade::{CombatEventFacade, CombatEventParam, CombatEventTag};

#[cfg(test)]
mod tests;
