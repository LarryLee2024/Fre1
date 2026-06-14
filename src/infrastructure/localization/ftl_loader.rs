//! FTL 文件加载工具
//!
//! 提供从文件系统加载 .ftl 翻译文件的工具函数。
//! 基于 std::fs 而非 AssetServer，适用于 Definition 类数据的一次性加载。

use std::fs;
use std::path::Path;

/// FTL 文件加载错误
#[derive(Debug, thiserror::Error)]
pub enum FtlLoadError {
    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("UTF-8 解码失败: {0}")]
    Utf8Error(String),

    #[error("IO 错误: {0}")]
    Io(String),
}

/// 从文件系统加载 .ftl 文件内容
pub fn load_ftl_file(path: &str) -> Result<String, FtlLoadError> {
    fs::read_to_string(path).map_err(|e| FtlLoadError::Io(format!("{}: {}", path, e)))
}

/// 发现指定目录下的所有 .ftl 文件
pub fn discover_ftl_files(dir: &str) -> Vec<String> {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Vec::new();
    }
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "ftl") {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            }
        }
    }
    files
}

/// 加载指定 locale 目录下的所有 FTL 文件并合并内容
pub fn load_locale_ftl_files(locale_dir: &str) -> Result<String, FtlLoadError> {
    let files = discover_ftl_files(locale_dir);
    let mut combined = String::new();

    for file in &files {
        let content = load_ftl_file(file)?;
        combined.push_str(&content);
        combined.push('\n');
    }

    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_ftl_files_nonexistent_dir() {
        let files = discover_ftl_files("/nonexistent/path");
        assert!(files.is_empty());
    }

    #[test]
    fn load_ftl_file_nonexistent() {
        let result = load_ftl_file("/nonexistent/file.ftl");
        assert!(result.is_err());
    }
}
