//! runtime — 跨领域运行时编排
//!
//! C3 Runtime 层：pipeline / scheduler / registry / command / replay 的编排底座。
//! 当前已实现：pipeline（执行管线）、scheduler（调度器）、registry（注册中心）、
//! command（命令层）、replay（回放）
//!
//! 详见 docs/01-architecture/README.md §3.2

pub mod command;
pub mod pipeline;
mod plugin;
pub mod registry;
pub mod replay;
pub mod scheduler;
pub use plugin::*;
