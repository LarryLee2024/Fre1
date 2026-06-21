//! Systems — 战斗领域系统
//!
//! 子模块：
//! - `turn_systems` — 回合状态机各阶段 System
//! - `effect_tick_system` — OnTurnEnd 驱动 Effect 计时推进
//! - `input_system` — 战斗回合输入处理
//! - `action_system` — 战斗动作执行（攻击/法术/伤害/死亡）
//! - `death_system` — HP 归零 safety net 检测

pub(crate) mod action_system;
pub(crate) mod death_system;
pub(crate) mod effect_tick_system;
pub(crate) mod input_system;
pub(crate) mod turn_systems;
