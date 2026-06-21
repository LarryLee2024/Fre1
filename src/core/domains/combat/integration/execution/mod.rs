//! 执行集成模块
//!
//! 包装执行能力以用于战斗伤害/治疗计算。

mod facade;

pub use facade::{CombatExecutionFacade, CombatExecutionParam};

#[cfg(test)]
mod tests;
