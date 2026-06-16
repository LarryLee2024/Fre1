//! 横切3: Tools — 开发工具层 (feature-gated)
//!
//! Debug 面板 / 性能分析 / 热重载控制台。
//! 仅 `#[cfg(feature = "dev")]` 构建中包含。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

mod dev_tools_plugin;

pub use dev_tools_plugin::DevToolsPlugin;
