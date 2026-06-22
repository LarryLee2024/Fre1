//! RON 文件发现 — 扫描 assets/config/ 目录结构
//!
//! # 日志模式
//!
//! 本文件属于**基础设施层直接日志（Pattern B）**，规则见 `content_plugin.rs` 模块说明。
//! 日志统一使用 `warn!(target: "content", "[Content] ...")` 格式。

use std::path::{Path, PathBuf};
use tracing::warn;

/// 发现的配置文件信息。
#[derive(Debug, Clone)]
pub struct ContentFile {
    /// 文件的完整路径。
    pub path: PathBuf,
    /// 相对于 config 根目录的路径。
    pub relative_path: PathBuf,
    /// 文件所属的桶名（从目录名推断）。
    pub bucket_name: String,
}

/// 扫描指定目录下的所有 .ron 文件。
///
/// 目录结构约定：
/// ```text
/// config/
/// ├── spells/        → bucket_name = "spells"
/// │   ├── fireball.ron
/// │   └── magic_missile.ron
/// ├── effects/
/// │   └── damage.ron
/// └── ...
/// ```
///
/// 返回所有发现的 .ron 文件信息。
pub fn discover_ron_files(config_root: &Path) -> Vec<ContentFile> {
    let mut files = Vec::new();

    if !config_root.exists() {
        return files;
    }

    discover_recursive(config_root, config_root, &mut files);
    files
}

/// 递归扫描目录。
fn discover_recursive(base: &Path, current: &Path, files: &mut Vec<ContentFile>) {
    let dir_str = current.display().to_string();
    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(target: "content", "[Content] 扫描配置目录失败: {} — {}", dir_str, e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            discover_recursive(base, &path, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            let relative = path.strip_prefix(base).unwrap_or(&path).to_path_buf();

            // 从第一级目录名推断 bucket_name
            let bucket_name = relative
                .components()
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .unwrap_or("unknown")
                .to_string();

            files.push(ContentFile {
                path,
                relative_path: relative,
                bucket_name,
            });
        }
    }
}
