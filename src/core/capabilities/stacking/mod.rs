//! stacking — 能力领域：堆叠规则
//!
//! 定义效果叠加的策略——同一效果多次作用时如何处理。
//! - 四种堆叠类型：None / Aggregate / RefreshDuration / Replace
//! - 同源/异源识别 + 溢出处理
//!
//! 详见 docs/02-domain/capabilities/stacking_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;
mod plugin;

pub use plugin::*;

#[cfg(test)]
mod tests;
