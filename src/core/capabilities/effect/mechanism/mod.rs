//! C2: 规则与系统层 — Effect 生命周期管理

// [ADR-045] pub(crate) — crate 内共享，测试可访问，外部不可访问
pub(crate) mod lifecycle;

pub use lifecycle::*;
