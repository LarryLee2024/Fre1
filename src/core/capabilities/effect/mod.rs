//! effect — Effect（效果）能力领域
//!
//! 效果运行时管理，负责效果的完整生命周期——从施加到移除。
//! 是技能流程的最后环节——Execution 计算出数值后以 Effect 的形式应用到目标。
//! 所有"结果"（伤害、治疗、Buff、Debuff）最终表现为 Effect。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（EffectStage, EffectDuration, EffectPeriod, EffectInstance）
//! - mechanism/:  生命周期管理（Lifecycle: 施加→持续/Tick→到期→移除）
//! - events/:     领域事件（EffectApplied, EffectRemoved, EffectTicked, EffectImmunityTriggered）
//!
//! 职责边界：
//! - 负责：效果生命周期管理（Applying→Active→Expiring→Removed）、周期 Tick、容器管理
//! - 不负责：叠加规则（Stacking）、属性修改（Modifier + Aggregator）、执行计算（Execution）
//!
//! 详见 docs/02-domain/capabilities/effect_domain.md
//! 详见 docs/04-data/capabilities/effect_schema.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
pub(crate) mod tests;
