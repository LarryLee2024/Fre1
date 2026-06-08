// 战斗模块：效果管线、伤害计算、战斗日志
// 合并了原 combat_event.rs、combat.rs、combat_log.rs

mod combat;
mod event;
mod log;
mod plugin;

// 公共 re-exports
pub use combat::*;
pub use event::*;
pub use log::*;
pub use plugin::BattlePlugin;
