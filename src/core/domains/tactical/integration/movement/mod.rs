//! 移动能力集成模块 — Tactical 域与 Capabilities 的移动相关对接。

pub mod facade;
pub mod system_param;
pub mod types;

pub use facade::*;
pub use system_param::MovementCapabilityParam;
pub use types::*;
