// RON 配置加载工具：消除注册表的重复加载代码
// 所有注册表共享相同的 load_from_dir / load_from_file 骨架

use ron::de::from_bytes;
use std::fs::{read, read_dir};

/// 从目录加载 RON 文件，每文件反序列化为单条记录
/// 返回 (成功加载的条目列表, 是否至少加载了一条)
pub fn load_dir_single<T>(dir: &str, type_name: &str) -> (Vec<T>, bool)
where
    T: serde::de::DeserializeOwned,
{
    let Ok(entries) = read_dir(dir) else {
        bevy::log::warn!("{}目录不存在: {}", type_name, dir);
        return (vec![], false);
    };

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ron") {
            match read(&path) {
                Ok(bytes) => match from_bytes::<T>(&bytes) {
                    Ok(item) => {
                        bevy::log::info!("加载{}: {:?}", type_name, path.file_stem());
                        items.push(item);
                    }
                    Err(e) => bevy::log::error!("解析{}文件 {:?} 失败: {}", type_name, path, e),
                },
                Err(e) => bevy::log::error!("读取{}文件 {:?} 失败: {}", type_name, path, e),
            }
        }
    }

    let loaded = !items.is_empty();
    if !loaded {
        bevy::log::warn!("{}目录为空或全部解析失败: {}", type_name, dir);
    }
    (items, loaded)
}

/// 从目录加载 RON 文件，每文件反序列化为数组
/// 返回 (成功加载的条目列表, 是否至少加载了一条)
pub fn load_dir_array<T>(dir: &str, type_name: &str) -> (Vec<T>, bool)
where
    T: serde::de::DeserializeOwned,
{
    let Ok(entries) = read_dir(dir) else {
        bevy::log::warn!("{}目录不存在: {}", type_name, dir);
        return (vec![], false);
    };

    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ron") {
            match read(&path) {
                Ok(bytes) => match from_bytes::<Vec<T>>(&bytes) {
                    Ok(file_items) => {
                        bevy::log::info!(
                            "加载{}: {:?} ({}条)",
                            type_name,
                            path.file_stem(),
                            file_items.len()
                        );
                        items.extend(file_items);
                    }
                    Err(e) => bevy::log::error!("解析{}文件 {:?} 失败: {}", type_name, path, e),
                },
                Err(e) => bevy::log::error!("读取{}文件 {:?} 失败: {}", type_name, path, e),
            }
        }
    }

    let loaded = !items.is_empty();
    if !loaded {
        bevy::log::warn!("{}目录为空或全部解析失败: {}", type_name, dir);
    }
    (items, loaded)
}

/// 从单个文件加载 RON，反序列化为数组
/// 返回 (成功加载的条目列表, 是否成功)
pub fn load_file_array<T>(path: &str, type_name: &str) -> (Vec<T>, bool)
where
    T: serde::de::DeserializeOwned,
{
    match read(path) {
        Ok(bytes) => match from_bytes::<Vec<T>>(&bytes) {
            Ok(items) => {
                bevy::log::info!("加载{}: {} 种", type_name, items.len());
                (items, true)
            }
            Err(e) => {
                bevy::log::error!("解析{}文件 {} 失败: {}", type_name, path, e);
                (vec![], false)
            }
        },
        Err(e) => {
            bevy::log::warn!("{}文件 {} 不存在: {}, 使用默认值", type_name, path, e);
            (vec![], false)
        }
    }
}
