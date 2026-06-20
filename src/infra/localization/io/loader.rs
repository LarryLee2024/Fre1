//! FTL 加载系统 — 从 assets/localization/ 加载所有 .ftl 文件
//!
//! 提供 `load_all_ftl_system` PreStartup 系统。
//!
//! 详见 `docs/03-technical/localization-design.md` §2

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;

use crate::infra::localization::foundation::{LocaleId, Pattern};
use crate::infra::localization::storage::LocalizationDatabase;

use super::parser::parse_ftl;

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
        let Ok(locale_id) = LocaleId::try_from(locale_name.as_str()) else {
            warn!(target: "localization", "[Localization] 未知 locale 目录: {}，跳过", locale_name);
            continue;
        };

        let mut locale_map = HashMap::new();
        let Ok(ftl_files) = std::fs::read_dir(&path) else {
            warn!(target: "localization", "[Localization] 无法读取 locale 目录：{:?}", path);
            continue;
        };

        for ftl_entry in ftl_files.flatten() {
            let ftl_path = ftl_entry.path();
            if ftl_path.extension().is_some_and(|e| e == "ftl") {
                let content = std::fs::read_to_string(&ftl_path)
                    .unwrap_or_else(|e| panic!("[Localization] 读取失败 {:?}: {}", ftl_path, e));
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
            locale_id,
            locale_map.len()
        );
        all.insert(locale_id, locale_map);
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
