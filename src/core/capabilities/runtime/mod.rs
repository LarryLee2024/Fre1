//! runtime — 跨领域运行时编排
//!
//! C3 Runtime 层：pipeline / scheduler / registry / command / replay 的编排底座。
//! 详见 docs/01-architecture/README.md §3.2

mod plugin;
pub use plugin::*;
