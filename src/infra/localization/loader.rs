//! FTL 解析器和加载系统
//!
//! 从 assets/localization/{locale}/*.ftl 加载文本数据，
//! 解析为扁平的 key → Pattern 映射写入 LocalizationDatabase。
//!
//! 详见 `docs/03-technical/localization-design.md` §2 + 附录 B

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use regex::Regex;

use super::database::LocalizationDatabase;
use super::database::Pattern;
use super::error::LocaleId;

/// 解析 .ftl 内容为扁平 key → Pattern 映射
///
/// 支持:
/// - Message ID: `-xxx-yyy = value`
/// - Attribute: `.desc = value`（需跟在 message ID 之后）
/// - 变量提取: `{$var}`
/// - 注释: `###` 开头的行
pub fn parse_ftl(content: &str) -> HashMap<String, Pattern> {
    let mut result = HashMap::new();
    // 正则对象仅在本函数内部创建，避免全局 lazy_static
    let id_re = Regex::new(r"^-([a-zA-Z0-9_-]+)\s*=\s*(.*)$").unwrap();
    let attr_re = Regex::new(r"^\s+\.([a-zA-Z0-9_-]+)\s*=\s*(.*)$").unwrap();
    let var_re = Regex::new(r"\{\$([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    let mut current_id: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // 跳过注释行（以 ### 开头）和空行
        if trimmed.starts_with("###") || trimmed.is_empty() {
            continue;
        }

        // Message ID: -xxx-yyy = value
        if let Some(caps) = id_re.captures(trimmed) {
            let raw_id = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str().trim();
            let key = raw_id.replace('-', ".");

            let vars: Vec<String> = var_re
                .captures_iter(value)
                .map(|c| c[1].to_string())
                .collect();

            result.insert(
                key.clone(),
                Pattern {
                    template: value.to_string(),
                    variables: vars,
                },
            );

            current_id = Some(key);
        }
        // Attribute: .xxx = value
        else if let Some(caps) = attr_re.captures(line)
            && let Some(ref base_key) = current_id
        {
            let attr_name = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str().trim();
            let key = format!("{}.{}", base_key, attr_name);

            let vars: Vec<String> = var_re
                .captures_iter(value)
                .map(|c| c[1].to_string())
                .collect();

            result.insert(
                key,
                Pattern {
                    template: value.to_string(),
                    variables: vars,
                },
            );
        }
    }

    result
}

/// 从 assets/localization/ 目录加载所有 locale 数据
fn load_all_locales() -> HashMap<LocaleId, HashMap<String, Pattern>> {
    let base = Path::new("assets/localization");
    let mut all = HashMap::new();

    let Ok(entries) = std::fs::read_dir(base) else {
        warn!(target: "localization", "[Localization] assets/localization/ 目录未找到 — 跳过加载");
        return all;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let locale_name = path.file_name().unwrap().to_string_lossy().to_string();

        let mut locale_map = HashMap::new();
        let Ok(ftl_files) = std::fs::read_dir(&path) else {
            warn!(target: "localization", "[Localization] 无法读取 locale 目录：{:?}", path);
            continue;
        };

        for ftl_entry in ftl_files.flatten() {
            let ftl_path = ftl_entry.path();
            if ftl_path.extension().is_some_and(|e| e == "ftl") {
                let content = std::fs::read_to_string(&ftl_path).unwrap_or_else(|e| {
                    panic!("[Localization] 读取失败 {:?}: {}", ftl_path, e)
                });
                let patterns = parse_ftl(&content);
                let count = patterns.len();
                locale_map.extend(patterns);
                info!(target: "localization", 
                    "[Localization] 从 {:?} 加载了 {} 个模式",
                    ftl_path, count
                );
            }
        }

        info!(target: "localization", 
            "[Localization] 加载了区域 '{}'，共 {} 个模式",
            locale_name,
            locale_map.len()
        );
        all.insert(locale_name, locale_map);
    }

    all
}

/// Startup System: 扫描 assets/localization/ 下所有 locale 目录和 .ftl 文件并加载
pub fn load_all_ftl_system(mut db: ResMut<LocalizationDatabase>) {
    info!(target: "localization", "[Localization] 开始加载 FTL 文件...");
    let all_locales = load_all_locales();

    let mut total_patterns = 0;
    for (locale, patterns) in all_locales {
        let count = patterns.len();
        db.load_patterns(&locale, patterns);
        total_patterns += count;
    }

    info!(target: "localization", 
        "[Localization] 加载了 {} 个区域共 {} 个模式。当前区域：{}",
        db.loaded_locales().len(),
        total_patterns,
        db.current_locale()
    );
}

// ── 热重载（仅在 debug 构建，非 wasm 平台） ──

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
    pub locale_dirs: Vec<(LocaleId, std::path::PathBuf)>,
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
                watcher
                    .watch(&path, RecursiveMode::Recursive)
                    .expect("Failed to watch locale directory");
                locale_dirs.push((locale_name, path));
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
    watcher: bevy::prelude::NonSendMut<LocaleWatcher>,
    mut db: ResMut<LocalizationDatabase>,
    mut cache: ResMut<super::cache::LocalizedTextCache>,
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
