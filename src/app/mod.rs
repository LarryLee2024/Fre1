//! 横切1: App — 启动装配层（Composition Root）
//!
//! 唯一知道所有层的入口点。
//! 根据 feature flag 启动 game/editor/headless。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

pub mod app_plugin;

pub use app_plugin::AppPlugin;
