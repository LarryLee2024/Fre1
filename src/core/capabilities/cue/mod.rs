//! cue — 能力领域：表现层信号
//!
//! 逻辑层通知表现层"该播放什么"的桥梁。
//! - 五种信号类型：VFX / SFX / Animation / Shake / Popup
//! - 四种触发时机：OnApply / OnTick / OnRemove / OnInterrupt
//! - 完全解耦：逻辑层不依赖表现层实现
//!
//! 详见 docs/02-domain/capabilities/cue_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;
mod plugin;

pub use plugin::*;

#[cfg(test)]
mod tests;
