//! localization — 国际化基础设施
//!
//! 提供 Fluent .ftl 文本的加载、解析、缓存、校验和渲染。
//! 属于 Infrastructure 层 (L2)。
//!
//! 模块结构：
//! - foundation/   纯类型，零 Bevy ECS 依赖（LocaleId, LocError, Pattern）
//! - storage/      ECS Resources（LocalizationDatabase, LocalizedTextCache）
//! - io/           文件 I/O 与解析（FTL parser, loader, hot-reload watcher）
//! - ui/           表现层（LocalizedText Component, render system）
//! - facade/       跨层编排（resolve_cached 组合 db + cache）
//! - validation/   启动校验与运行时审计（validator, audit）
//!
//! 详见 `docs/03-technical/localization-design.md`
//! 详见 ADR-053

mod facade;
mod foundation;
mod io;
mod plugin;
mod storage;
mod ui;
mod validation;

pub mod generated {
    include!("generated/keys.rs");
}

#[cfg(test)]
mod tests;

pub use foundation::{LocError, LocaleId};
pub use plugin::LocalizationPlugin;
pub use storage::LocalizationDatabase;
pub use ui::LocalizedText;
