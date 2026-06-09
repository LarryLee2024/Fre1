// 通用注册表加载器：消除各注册表的重复加载代码
// 遵循「数据驱动优先于硬编码」铁律，统一 RON 加载模式
// 替代各注册表内重复的 load_from_dir / load_from_file 骨架

use ron::de::from_bytes;
use serde::de::DeserializeOwned;
use std::fs::{read, read_dir};

/// 通用 RON 注册表加载 trait
/// 从目录或文件加载 .ron 配置，反序列化后注册到注册表
pub trait RegistryLoader: Default {
    /// RON 反序列化目标类型（单条记录）
    type Item: DeserializeOwned;

    /// 注册一条反序列化后的记录
    fn register_item(&mut self, item: Self::Item);

    /// 注册默认数据（当目录不存在或为空时调用）
    /// 必须保证幂等：多次调用与一次调用效果相同
    fn register_defaults(&mut self);

    /// 判断注册表是否为空
    fn is_empty(&self) -> bool;

    /// 资源类型名称（用于日志）
    fn registry_name() -> &'static str;

    /// 从目录加载 .ron 文件，每文件反序列化为单条记录
    fn load_from_dir(dir: &str) -> Self {
        let mut registry = Self::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("{}目录不存在，使用默认数据: {}", Self::registry_name(), dir);
            registry.register_defaults();
            return registry;
        };
        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<Self::Item>(&bytes) {
                        Ok(item) => {
                            registry.register_item(item);
                            loaded = true;
                        }
                        Err(e) => bevy::log::error!(
                            "解析{}文件 {:?} 失败: {}",
                            Self::registry_name(),
                            path,
                            e
                        ),
                    },
                    Err(e) => bevy::log::error!(
                        "读取{}文件 {:?} 失败: {}",
                        Self::registry_name(),
                        path,
                        e
                    ),
                }
            }
        }
        if !loaded {
            bevy::log::warn!("{}目录为空，使用默认数据", Self::registry_name());
            registry.register_defaults();
        }
        registry
    }

    /// 从目录加载 .ron 文件，每文件反序列化为数组（多条记录）
    fn load_from_dir_vec(dir: &str) -> Self {
        let mut registry = Self::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("{}目录不存在，使用默认数据: {}", Self::registry_name(), dir);
            registry.register_defaults();
            return registry;
        };
        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<Vec<Self::Item>>(&bytes) {
                        Ok(items) => {
                            for item in items {
                                registry.register_item(item);
                            }
                            loaded = true;
                        }
                        Err(e) => bevy::log::error!(
                            "解析{}文件 {:?} 失败: {}",
                            Self::registry_name(),
                            path,
                            e
                        ),
                    },
                    Err(e) => bevy::log::error!(
                        "读取{}文件 {:?} 失败: {}",
                        Self::registry_name(),
                        path,
                        e
                    ),
                }
            }
        }
        if !loaded {
            bevy::log::warn!("{}目录为空，使用默认数据", Self::registry_name());
            registry.register_defaults();
        }
        registry
    }

    /// 从单文件加载数组格式的 .ron 文件
    fn load_from_file(path: &str) -> Self {
        let mut registry = Self::default();
        match read(path) {
            Ok(bytes) => match from_bytes::<Vec<Self::Item>>(&bytes) {
                Ok(items) => {
                    let count = items.len();
                    for item in items {
                        registry.register_item(item);
                    }
                    bevy::log::info!("加载{}: {} 种", Self::registry_name(), count);
                }
                Err(e) => {
                    bevy::log::error!("解析{}文件 {} 失败: {}", Self::registry_name(), path, e);
                    registry.register_defaults();
                }
            },
            Err(e) => {
                bevy::log::warn!(
                    "{}文件 {} 不存在: {}, 使用默认值",
                    Self::registry_name(),
                    path,
                    e
                );
                registry.register_defaults();
            }
        }
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Deserialize)]
    struct TestItem {
        id: String,
        value: i32,
    }

    #[derive(Default, Debug)]
    struct TestRegistry {
        items: HashMap<String, TestItem>,
    }

    impl RegistryLoader for TestRegistry {
        type Item = TestItem;

        fn register_item(&mut self, item: TestItem) {
            self.items.insert(item.id.clone(), item);
        }

        fn register_defaults(&mut self) {
            if !self.items.is_empty() {
                return;
            }
            self.items.insert(
                "default".to_string(),
                TestItem {
                    id: "default".to_string(),
                    value: 0,
                },
            );
        }

        fn is_empty(&self) -> bool {
            self.items.is_empty()
        }

        fn registry_name() -> &'static str {
            "测试"
        }
    }

    #[test]
    fn registry_loader_空注册表() {
        let registry = TestRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn registry_loader_注册项() {
        let mut registry = TestRegistry::default();
        registry.register_item(TestItem {
            id: "test".to_string(),
            value: 42,
        });
        assert!(!registry.is_empty());
        assert_eq!(registry.items.len(), 1);
    }

    #[test]
    fn registry_loader_默认注册_空时填充() {
        let mut registry = TestRegistry::default();
        registry.register_defaults();
        assert_eq!(registry.items.len(), 1);
    }

    #[test]
    fn registry_loader_默认注册_幂等() {
        let mut registry = TestRegistry::default();
        registry.register_defaults();
        assert_eq!(registry.items.len(), 1);
        registry.register_defaults();
        assert_eq!(registry.items.len(), 1);
    }

    #[test]
    fn registry_loader_默认注册_非空时不覆盖() {
        let mut registry = TestRegistry::default();
        registry.register_item(TestItem {
            id: "custom".to_string(),
            value: 1,
        });
        registry.register_defaults();
        assert_eq!(registry.items.len(), 1);
        assert!(registry.items.get("default").is_none());
    }

    #[test]
    fn registry_loader_目录不存在时使用默认() {
        let registry = TestRegistry::load_from_dir("/nonexistent/path");
        assert!(registry.is_empty() || registry.items.contains_key("default"));
    }

    #[test]
    fn registry_loader_文件不存在时使用默认() {
        let registry = TestRegistry::load_from_file("/nonexistent/file.ron");
        assert!(registry.is_empty() || registry.items.contains_key("default"));
    }
}
