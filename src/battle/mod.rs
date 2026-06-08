// 战斗模块：效果管线、伤害计算、战斗日志

mod combat;
pub mod pipeline;
mod log;
mod plugin;

// 公共 re-exports
pub use combat::*;
pub use pipeline::{CombatIntent, PrevPosition};
pub use log::*;
pub use plugin::BattlePlugin;
