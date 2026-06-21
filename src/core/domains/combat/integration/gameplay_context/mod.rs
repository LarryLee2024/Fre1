//! GameplayContext 集成模块
//!
//! 封装 gameplay_context 能力，用于战斗行动上下文构建。

mod facade;

pub use facade::{CombatContextFacade, CombatContextParam};

#[cfg(test)]
mod tests;
