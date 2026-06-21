//! 聚合器集成模块
//!
//! 包装聚合器能力以用于战斗属性重新计算。

mod facade;

pub use facade::{CombatAggregatorFacade, CombatAggregatorParam};

#[cfg(test)]
mod tests;
