//! stacking — 能力领域：堆叠规则
//!
//! 定义效果叠加的策略——同一效果多次作用时如何处理。
//! - 四种堆叠类型：None / Aggregate / RefreshDuration / Replace
//! - 同源/异源识别 + 溢出处理
//!
//! 详见 docs/02-domain/stacking_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;
mod plugin;

pub use plugin::*;

#[cfg(test)]
mod tests;
