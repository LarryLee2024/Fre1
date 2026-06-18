//! Trigger Integration Module

mod facade;
mod system_param;

pub use facade::{CombatTriggerFacade, CombatTriggerType};
pub use system_param::CombatTriggerParam;

#[cfg(test)]
mod tests;
