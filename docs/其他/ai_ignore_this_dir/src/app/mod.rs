//! App 层：游戏启动与装配
//!
//! Layer 1 职责：组装整个游戏，只注册，不含逻辑。

/// 游戏错误事件（GameErrorEvent）定义
pub mod error_event;
/// 错误监控系统（error_monitor）
pub mod error_monitor;
pub mod plugin;
pub use plugin::AppPlugin;
