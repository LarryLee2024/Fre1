//! observers — 领域事件日志 Observer 集合
//!
//! 每个子模块监听一组领域事件，生成结构化日志。
//! 所有 Observer 只读事件，不修改任何领域状态。
//!
//! 日志格式：每个 Observer 使用 `#[instrument]` 生成 span，
//! 内部通过 `ctx.log_info/log_warn` 输出结构化事件。
//! 同时通过 `telemetry::emit()` 触发度量计数。

pub(crate) mod ability_logger;
pub(crate) mod battle_logger;
pub(crate) mod camp_rest_logger;
pub(crate) mod content_logger;
pub(crate) mod crafting_logger;
pub(crate) mod economy_logger;
pub(crate) mod effect_logger;
pub(crate) mod faction_logger;
pub(crate) mod inventory_logger;
pub(crate) mod narrative_logger;
pub(crate) mod party_logger;
pub(crate) mod progression_logger;
pub(crate) mod quest_logger;
pub(crate) mod reaction_logger;
pub(crate) mod spell_logger;
pub(crate) mod summon_logger;
pub(crate) mod tactical_logger;
pub(crate) mod terrain_logger;
pub(crate) mod turn_logger;
