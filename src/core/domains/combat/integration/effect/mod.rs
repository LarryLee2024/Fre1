//! 效果能力集成模块 — Combat 域与 Effect Capability 的对接。
//!
//! 此子模块是 Combat 域调用 Effect 能力（`ActiveEffectContainer`、`tick_durations`、
//! `expire_effects` 等）的唯一入口。
//!
//! # 子模块
//!
//! - `facade` — 业务语义 API（唯一访问 Effect 内部类型的地方）
//! - `types` — Combat 视图类型
//! - `system_param` — Bevy SystemParam（Systems 通过此 param 交互）
//!
//! # 参考
//!
//! - ADR-024 §2 — Effect 集成模块详细设计

pub mod facade;
pub mod system_param;
pub mod types;

pub use facade::*;
pub use system_param::EffectTickParam;
pub use types::*;

#[cfg(test)]
mod tests;
