//! runtime — 跨领域运行时编排
//!
//! C3 Runtime 层：pipeline / scheduler / registry / command / replay 的编排底座。
//! 当前已实现：pipeline（执行管线）、scheduler（调度器）、registry（注册中心）、
//! command（命令层）、replay（回放）
//!
//! 详见 docs/01-architecture/README.md §3.2

// [ADR-045] pub(crate) — runtime 子模块，crate 内共享，外部不可访问
pub(crate) mod command;
pub(crate) mod pipeline;
mod plugin;
pub(crate) mod registry;
pub(crate) mod replay;
pub(crate) mod scheduler;
pub use plugin::*;
