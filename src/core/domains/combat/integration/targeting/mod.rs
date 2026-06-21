//! 目标选择集成模块
//!
//! 包装目标选择能力以用于战斗特定的目标选择。

mod facade;

pub use facade::{CombatTargetingFacade, CombatTargetingParam};

#[cfg(test)]
mod tests;
