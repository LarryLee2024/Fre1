//! observers — 领域事件日志 Observer 集合
//!
//! 每个子模块监听一组领域事件，生成结构化日志。
//! 所有 Observer 只读事件，不修改任何领域状态。

pub(crate) mod ability_logger;
pub(crate) mod battle_logger;
pub(crate) mod content_logger;
pub(crate) mod effect_logger;
pub(crate) mod quest_logger;
pub(crate) mod spell_logger;
pub(crate) mod turn_logger;
