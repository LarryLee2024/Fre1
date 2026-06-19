//! localization — 国际化基础设施
//!
//! 提供 Fluent .ftl 文本的加载、解析、缓存、校验和渲染。
//! 属于 Infrastructure 层 (L2)。
//!
//! 详见 `docs/03-technical/localization-design.md`
//! 详见 ADR-053

mod audit;
mod cache;
mod components;
mod database;
mod error;
mod loader;
mod plugin;
mod validator;

pub mod generated {
    include!("generated/keys.rs");
}

pub use components::LocalizedText;
pub use database::LocalizationDatabase;
pub use error::LocError;
pub use plugin::LocalizationPlugin;
