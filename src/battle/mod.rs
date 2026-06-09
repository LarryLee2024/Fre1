// 战斗模块：效果管线、伤害计算、战斗日志、战斗事件

mod combat;
mod events;
mod log;
mod pipeline;
mod plugin;
mod record;

// 公共 re-exports
pub use combat::*;
pub use events::*;
pub use log::*;
pub use pipeline::{CombatIntent, PrevPosition};
pub use plugin::BattlePlugin;
pub use record::*;
