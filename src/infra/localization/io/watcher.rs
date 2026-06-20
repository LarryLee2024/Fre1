//! FTL 热重载 — 文件系统监控与运行时重载
//!
//! 仅在 debug 构建且非 wasm 平台时编译。

use std::path::PathBuf;

use bevy::prelude::*;

use crate::infra::localization::foundation::LocaleId;
use crate::infra::localization::storage::{LocalizationDatabase, LocalizedTextCache};

use super::parser::parse_ftl;

/// 资源: 文件系统监控器
///
/// 使用 `NonSend` 因为 `mpsc::Receiver` 不是 `Sync`。
/// `watcher` 字段不直接读取，但必须保持存活以持续监控文件变化。
#[cfg(debug_assertions)]
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub struct LocaleWatcher {
    pub watcher: notify::RecommendedWatcher,
    pub receiver: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    pub locale_dirs: Vec<(LocaleId, PathBuf)>,
}

/// 创建文件监控器（在 Plugin::build() 中调用）
#[cfg(debug_assertions)]
#[cfg(not(target_arch = "wasm32"))]
pub fn create_locale_watcher() -> LocaleWatcher {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            let _ = tx.send(res);
        },
        Config::default(),
    )
    .expect("[Localization] 创建文件监听器失败");

    let base = std::path::Path::new("assets/localization");
    let mut locale_dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let locale_name = path.file_name().unwrap().to_string_lossy().to_string();
                if let Ok(locale_id) = LocaleId::try_from(locale_name.as_str()) {
                    watcher
                        .watch(&path, RecursiveMode::Recursive)
                        .expect("Failed to watch locale directory");
                    locale_dirs.push((locale_id, path));
                }
            }
        }
    }

    LocaleWatcher {
        watcher,
        receiver: rx,
        locale_dirs,
    }
}

/// Update System: 检查文件变化并重载
#[cfg(debug_assertions)]
#[cfg(not(target_arch = "wasm32"))]
pub fn hot_reload_system(
    watcher: NonSendMut<LocaleWatcher>,
    mut db: ResMut<LocalizationDatabase>,
    mut cache: ResMut<LocalizedTextCache>,
) {
    while let Ok(Ok(event)) = watcher.receiver.try_recv() {
        use notify::EventKind;

        let paths = match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => event.paths,
            _ => continue,
        };

        for path in &paths {
            if let Some(ext) = path.extension()
                && ext == "ftl"
            {
                let Some((locale, _)) = watcher
                    .locale_dirs
                    .iter()
                    .find(|(_, dir)| path.starts_with(dir))
                else {
                    continue;
                };

                let Ok(content) = std::fs::read_to_string(path) else {
                    continue;
                };
                if content.trim().is_empty() {
                    continue;
                }

                let patterns = parse_ftl(&content);
                let count = patterns.len();
                db.load_patterns(locale, patterns);

                info!(target: "localization",
                    "[Localization] 热重载了 {} 个模式从 {:?}",
                    count, path
                );

                cache.clear();
            }
        }
    }
}
