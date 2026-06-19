//! Systems — 战斗领域系统
//!
//! 子模块：
//! - `turn_systems` — 回合状态机各阶段 System
//! - `effect_tick_system` — OnTurnEnd 驱动 Effect 计时推进
//! - `input_system` — 战斗回合输入处理

pub(crate) mod effect_tick_system;
pub(crate) mod input_system;
pub(crate) mod turn_systems;
