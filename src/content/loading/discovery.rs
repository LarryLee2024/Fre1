//! RON 文件发现 — 扫描 assets/config/ 目录结构

use std::path::{Path, PathBuf};

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
    let entries = match std::fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discover_empty_dir() {
        let dir = std::env::temp_dir().join("fre_test_discover_empty");
        let _ = fs::create_dir_all(&dir);
        let files = discover_ron_files(&dir);
        assert!(files.is_empty());
    }

    #[test]
    fn discover_nonexistent_dir() {
        let dir = Path::new("/nonexistent/path/config");
        let files = discover_ron_files(dir);
        assert!(files.is_empty());
    }
}
