//! C2: 规则与系统层 — ECS 组件、注册表、System

mod components;
// [ADR-045] pub(crate) — crate 内共享，测试可访问，外部不可访问
pub(crate) mod lifecycle;
pub(crate) mod systems;

pub use components::*;
pub use lifecycle::*;
