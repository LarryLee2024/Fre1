//! aggregator — 属性聚合管线能力领域
//!
//! 将 Attribute 的 BaseValue 与所有活跃 Modifier 按规则计算为 FinalValue。
//! 四阶段管线: Add → Multiply → Override → Clamp
//!
//! 详见 docs/02-domain/aggregator_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;
