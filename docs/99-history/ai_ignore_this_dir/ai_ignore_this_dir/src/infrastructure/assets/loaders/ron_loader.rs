/// RON 文件加载工具
///
/// 提供从文件系统加载 .ron 配置文件的通用工具函数。
/// 基于 std::fs 而非 AssetServer，适用于编译时确定性数据加载。
///
/// 🟥 注意：此工具适用于 Definition 类数据（如技能、Buff 配置）的一次性加载。
/// 运行时动态资源应使用 Bevy AssetServer。
use std::fs;
use std::path::Path;

use super::super::asset_error::AssetError;

/// 发现指定目录下的所有 .ron 文件
pub fn discover_ron_files(dir: &str) -> Vec<String> {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return Vec::new();
    }
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "ron") {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            }
        }
    }
    files
}

/// 加载并解析单个 RON 文件
pub fn load_ron_file<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, AssetError> {
    let content = fs::read_to_string(path).map_err(|_e| AssetError::FileNotFound {
        path: path.to_string(),
    })?;
    let value = ron::from_str(&content).map_err(|e| AssetError::ParseError {
        path: path.to_string(),
        detail: e.to_string(),
    })?;
    Ok(value)
}
