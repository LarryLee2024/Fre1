//! storage — 本地化数据的 ECS Resource。
//!
//! 包含 LocalizationDatabase（核心数据）和 LocalizedTextCache（运行时缓存）。

pub(crate) mod cache;
pub(crate) mod database;

pub use cache::{LocalizedTextCache, detect_locale_change_and_clear_cache};
pub use database::LocalizationDatabase;
