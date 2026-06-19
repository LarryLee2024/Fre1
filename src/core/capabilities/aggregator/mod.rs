//! aggregator — 属性聚合管线能力领域
//!
//! 将 Attribute 的 BaseValue 与所有活跃 Modifier 按规则计算为 FinalValue。
//! 四阶段管线: Add → Multiply → Override → Clamp
//!
//! 详见 docs/02-domain/capabilities/aggregator_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
