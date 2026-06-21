//! io — .ftl 本地化文件的文件 I/O 和解析。
//!
//! 包含 FTL 解析器、文件加载系统和热重载监视器。

pub(crate) mod loader;
pub(crate) mod parser;
pub(crate) mod watcher;

pub use loader::load_all_ftl_system;
pub use parser::parse_ftl;

#[cfg(debug_assertions)]
#[cfg(not(target_arch = "wasm32"))]
pub use watcher::{LocaleWatcher, create_locale_watcher, hot_reload_system};
