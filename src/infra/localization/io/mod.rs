//! io — File I/O and parsing for .ftl localization files.
//!
//! Contains the FTL parser, file loading system, and hot-reload watcher.

pub(crate) mod loader;
pub(crate) mod parser;
pub(crate) mod watcher;

pub use loader::load_all_ftl_system;
pub use parser::parse_ftl;

#[cfg(debug_assertions)]
#[cfg(not(target_arch = "wasm32"))]
pub use watcher::{LocaleWatcher, create_locale_watcher, hot_reload_system};
