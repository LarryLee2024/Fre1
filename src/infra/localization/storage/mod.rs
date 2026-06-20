//! storage — ECS Resources for localization data.
//!
//! Contains LocalizationDatabase (core data) and LocalizedTextCache (runtime cache).

pub(crate) mod cache;
pub(crate) mod database;

pub use cache::{LocalizedTextCache, detect_locale_change_and_clear_cache};
pub use database::LocalizationDatabase;
