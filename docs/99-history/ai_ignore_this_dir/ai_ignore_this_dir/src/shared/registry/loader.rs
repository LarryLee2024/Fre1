//! RON 文件加载器（ADR-030 §2.1）
//!
//! 提供从 RON 文件系统加载数据的底层工具函数，
//! 供 [`LoadableRegistry`](super::LoadableRegistry) 的默认实现使用。
//!
//! # 功能
//! - `load_ron_file` — 加载单文件、单 Def 或数组 Def
//! - `load_ron_dir` — 加载目录下所有 `.ron` 文件（每个文件一条记录）
//! - `load_ron_dir_vec` — 加载目录下所有 `.ron` 文件（每个文件为数组格式）

use ron::de::from_bytes;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

/// 注册表加载相关错误。
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// 文件 I/O 错误（路径不存在、无权访问等）
    #[error("Registry load I/O error at {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    /// RON 反序列化错误（语法错误、类型不匹配等）
    #[error("Registry load RON parse error at {path}: {source}")]
    Ron {
        path: String,
        source: ron::error::SpannedError,
    },
}

impl LoadError {
    fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        LoadError::Io {
            path: path.into(),
            source,
        }
    }

    fn ron(path: impl Into<String>, source: ron::error::SpannedError) -> Self {
        LoadError::Ron {
            path: path.into(),
            source,
        }
    }
}

/// 从单文件加载并反序列化为指定类型。
///
/// 适用于单个 Def 或 `Vec<Def>` 等任意 `DeserializeOwned` 类型。
///
/// # 错误
/// - 文件不存在或不可读 → `LoadError::Io`
/// - RON 格式错误 → `LoadError::Ron`
pub fn load_ron_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, LoadError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|e| LoadError::io(path.display().to_string(), e))?;
    from_bytes(&bytes).map_err(|e| LoadError::ron(path.display().to_string(), e))
}

/// 从目录加载所有 `.ron` 文件，每个文件反序列化为一条记录。
///
/// 忽略隐藏文件（以 `.` 开头）和非 `.ron` 后缀的文件。
///
/// # 错误
/// - 目录不存在 → 返回空 Vec（不是错误 — 与旧行为一致）
/// - 目录存在但无法遍历 → 返回错误
/// - 任何 `.ron` 文件解析失败 → 返回首个错误
pub fn load_ron_dir<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<Vec<T>, LoadError> {
    let path = path.as_ref();
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()), // 目录不存在 → 空结果，不是错误
    };

    let mut results = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| LoadError::io(path.display().to_string(), e))?;
        let file_path = entry.path();

        // 忽略隐藏文件（以 `.` 开头）和非 `.ron` 后缀
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.starts_with('.') || !file_name.ends_with(".ron") {
            continue;
        }

        let item: T = load_ron_file(&file_path)?;
        results.push(item);
    }

    Ok(results)
}

/// 从目录加载所有 `.ron` 文件，每个文件反序列化为 `Vec<T>`（数组格式）。
///
/// 忽略隐藏文件和非 `.ron` 后缀的文件。
pub fn load_ron_dir_vec<T: DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<Vec<Vec<T>>, LoadError> {
    let path = path.as_ref();
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };

    let mut results = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| LoadError::io(path.display().to_string(), e))?;
        let file_path = entry.path();

        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name.starts_with('.') || !file_name.ends_with(".ron") {
            continue;
        }

        let items: Vec<T> = load_ron_file(&file_path)?;
        results.push(items);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Write;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestItem {
        id: String,
        value: i32,
    }

    fn create_temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    // ── load_ron_file ──

    #[test]
    fn load_ron_file_single_item() {
        let dir = create_temp_dir("loader_test_file_single");
        let file_path = dir.join("data.ron");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"(id: \"test\", value: 42)").unwrap();

        let item: TestItem = load_ron_file(&file_path).unwrap();
        assert_eq!(item.id, "test");
        assert_eq!(item.value, 42);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_ron_file_vec() {
        let dir = create_temp_dir("loader_test_file_vec");
        let file_path = dir.join("data.ron");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"[(id: \"a\", value: 1), (id: \"b\", value: 2)]")
            .unwrap();

        let items: Vec<TestItem> = load_ron_file(&file_path).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "a");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_ron_file_nonexistent_returns_error() {
        let result: Result<TestItem, _> = load_ron_file("/tmp/nonexistent_file_xyz123.ron");
        assert!(result.is_err());
        match result.unwrap_err() {
            LoadError::Io { .. } => {} // expected
            other => panic!("Expected Io error, got: {other}"),
        }
    }

    #[test]
    fn load_ron_file_malformed_ron_returns_error() {
        let dir = create_temp_dir("loader_test_malformed");
        let file_path = dir.join("bad.ron");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"this is not valid ron {{{{").unwrap();

        let result: Result<TestItem, _> = load_ron_file(&file_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            LoadError::Ron { .. } => {} // expected
            other => panic!("Expected Ron error, got: {other}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── load_ron_dir ──

    #[test]
    fn load_ron_dir_multiple_files() {
        let dir = create_temp_dir("loader_test_dir_multi");
        let mut f1 = std::fs::File::create(dir.join("a.ron")).unwrap();
        f1.write_all(b"(id: \"alpha\", value: 10)").unwrap();
        let mut f2 = std::fs::File::create(dir.join("b.ron")).unwrap();
        f2.write_all(b"(id: \"beta\", value: 20)").unwrap();

        let items: Vec<TestItem> = load_ron_dir(&dir).unwrap();
        assert_eq!(items.len(), 2);

        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"alpha"));
        assert!(ids.contains(&"beta"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_ron_dir_ignores_non_ron_and_hidden() {
        let dir = create_temp_dir("loader_test_ignore");
        let mut f1 = std::fs::File::create(dir.join("data.ron")).unwrap();
        f1.write_all(b"(id: \"only\", value: 1)").unwrap();
        let mut f2 = std::fs::File::create(dir.join("notes.txt")).unwrap();
        f2.write_all(b"hello").unwrap();
        let mut f3 = std::fs::File::create(dir.join(".hidden.ron")).unwrap();
        f3.write_all(b"(id: \"secret\", value: 99)").unwrap();

        let items: Vec<TestItem> = load_ron_dir(&dir).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "only");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_ron_dir_nonexistent_returns_empty() {
        let items: Vec<TestItem> = load_ron_dir("/tmp/nonexistent_dir_xyz456").unwrap();
        assert!(items.is_empty());
    }
}
