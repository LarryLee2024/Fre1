//! 统一 Registry 基础设施（ADR-030 §2）
//!
//! 提供三层 Registry trait 族：
//! - [`Registry`] — 只读查询接口
//! - [`LoadableRegistry`] — 可加载注册表（从 RON 反序列化构建）
//! - [`ValidatableRegistry`] — 支持自校验的注册表
//!
//! 以及 DAG 初始化系统（[`RegistryInitStage`], [`RegistryPlugin`]）。
//!
//! # 类型安全
//!
//! 每个 Registry 绑定具体的 Key 类型，在编译期防止 ID 错配：
//! ```ignore
//! let reg: AbilityRegistry = …;
//! let a: &AbilityData = reg.get(&ability_id); // ✅
//! let b: &AbilityData = reg.get(&effect_id);  // ❌ 编译错误
//! ```

pub mod init;
pub mod loader;
pub mod validatable;

pub use init::{RegistryInitStage, RegistryPlugin};
pub use loader::LoadError;
pub use validatable::{ValidatableRegistry, ValidationError};

use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::hash::Hash;

/// 所有运行时 Registry 的统一只读查询接口。
///
/// 每个 Registry 实现绑定具体的 `Key`（强类型 ID）和 `Data`（运行时数据）。
/// 加载完成后不可变，只读查询。
pub trait Registry: Send + Sync + 'static {
    /// 注册表键类型（强类型 ID，如 `AbilityId`）
    type Key: Display + Hash + Eq + 'static;

    /// 运行时数据类型
    type Data: 'static;

    /// 已注册项数量
    fn len(&self) -> usize;

    /// 注册表是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 按 ID 查询数据
    fn get(&self, key: &Self::Key) -> Option<&Self::Data>;

    /// 检查 ID 是否已注册
    fn contains(&self, key: &Self::Key) -> bool {
        self.get(key).is_some()
    }

    /// 返回所有已注册的 Key
    fn keys(&self) -> Vec<&Self::Key>;

    /// 遍历所有 (Key, Data) 条目
    fn iter(&self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Data)> + '_>;
}

/// 支持从 RON 文件加载的注册表。
///
/// 提供 `load_from_dir` 和 `load_from_file` 的默认实现，
/// 子类只需实现 `register_def`（Def → (Key, Data) 转换 + 注册）。
pub trait LoadableRegistry: Registry + Default + Sized {
    /// RON 反序列化的中间定义类型（如 `EquipmentDef`）
    type Def: DeserializeOwned + 'static;

    /// 加载错误类型。
    ///
    /// 必须实现 `From<LoadError>` 以支持默认的 RON 加载实现。
    type Error: std::error::Error + From<LoadError>;

    /// 将一条 Def 转换并注册到注册表中。
    ///
    /// # 实现要求
    /// - 从 `def` 中提取 `Key`，并将业务数据转换为 `Self::Data`
    /// - 将 `(key, data)` 插入内部存储
    /// - 发现重复 key 时宜返回 `Err`
    fn register_def(&mut self, def: Self::Def) -> Result<(), Self::Error>;

    /// 从目录加载 .ron 文件，每个文件反序列化为一条 `Def`。
    fn load_from_dir(path: impl AsRef<std::path::Path>) -> Result<Self, Self::Error> {
        let mut registry = Self::default();
        let defs: Vec<Self::Def> = loader::load_ron_dir(path.as_ref())?;
        for def in defs {
            registry.register_def(def)?;
        }
        Ok(registry)
    }

    /// 从目录加载 .ron 文件，每个文件反序列化为 `Vec<Def>`（数组格式）。
    fn load_from_dir_vec(path: impl AsRef<std::path::Path>) -> Result<Self, Self::Error> {
        let mut registry = Self::default();
        let def_groups: Vec<Vec<Self::Def>> = loader::load_ron_dir_vec(path.as_ref())?;
        for defs in def_groups {
            for def in defs {
                registry.register_def(def)?;
            }
        }
        Ok(registry)
    }

    /// 从单文件加载数组格式的 .ron 文件。
    fn load_from_file(path: impl AsRef<std::path::Path>) -> Result<Self, Self::Error> {
        let mut registry = Self::default();
        let defs: Vec<Self::Def> = loader::load_ron_file(path.as_ref())?;
        for def in defs {
            registry.register_def(def)?;
        }
        Ok(registry)
    }
}

/// 支持从 RON 文件加载的注册表（单条 Def 模式）。
///
/// 与 [`LoadableRegistry`] 的区别：
/// - 此 trait 的 `load_from_file` 反序列化为单条 `Def`（而非 `Vec<Def>`）
/// - 适用于每个 RON 文件只包含一个记录的配置
pub trait LoadableSingleRegistry: Registry + Default + Sized {
    /// RON 反序列化的中间定义类型
    type Def: DeserializeOwned + 'static;

    /// 加载错误类型。
    ///
    /// 必须实现 `From<LoadError>` 以支持默认的 RON 加载实现。
    type Error: std::error::Error + From<LoadError>;

    /// 将一条 Def 转换并注册
    fn register_def(&mut self, def: Self::Def) -> Result<(), Self::Error>;

    /// 从目录加载 .ron 文件，每个文件一条记录
    fn load_from_dir(path: impl AsRef<std::path::Path>) -> Result<Self, Self::Error> {
        let mut registry = Self::default();
        let defs: Vec<Self::Def> = loader::load_ron_dir(path.as_ref())?;
        for def in defs {
            registry.register_def(def)?;
        }
        Ok(registry)
    }

    /// 从单文件加载（反序列化为单条 Def）
    fn load_from_file(path: impl AsRef<std::path::Path>) -> Result<Self, Self::Error> {
        let mut registry = Self::default();
        let def: Self::Def = loader::load_ron_file(path.as_ref())?;
        registry.register_def(def)?;
        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    // ── Test helpers ──

    /// 测试用 Def 类型
    #[derive(Debug, Clone, Deserialize)]
    struct TestDef {
        id: String,
        value: i32,
    }

    /// 测试用 Data 类型
    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: i32,
    }

    /// 测试用 Registry：包装 HashMap
    #[derive(Default, Debug)]
    struct TestRegistry {
        items: HashMap<String, TestData>,
    }

    impl Registry for TestRegistry {
        type Key = String;
        type Data = TestData;

        fn len(&self) -> usize {
            self.items.len()
        }

        fn get(&self, key: &Self::Key) -> Option<&Self::Data> {
            self.items.get(key)
        }

        fn keys(&self) -> Vec<&Self::Key> {
            self.items.keys().collect()
        }

        fn iter(&self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Data)> + '_> {
            Box::new(self.items.iter())
        }
    }

    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    impl From<super::LoadError> for TestError {
        fn from(e: super::LoadError) -> Self {
            TestError(e.to_string())
        }
    }

    impl LoadableRegistry for TestRegistry {
        type Def = TestDef;
        type Error = TestError;

        fn register_def(&mut self, def: Self::Def) -> Result<(), Self::Error> {
            if self.items.contains_key(&def.id) {
                return Err(TestError(format!("Duplicate key: {}", def.id)));
            }
            self.items.insert(def.id, TestData { value: def.value });
            Ok(())
        }
    }

    // ── Registry trait tests ──

    #[test]
    fn registry_empty_by_default() {
        let reg = TestRegistry::default();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn registry_get_after_register() {
        let mut reg = TestRegistry::default();
        reg.register_def(TestDef {
            id: "test_1".into(),
            value: 42,
        })
        .unwrap();

        assert_eq!(reg.len(), 1);
        assert!(!reg.is_empty());
        assert!(reg.contains(&"test_1".to_string()));
        assert_eq!(reg.get(&"test_1".to_string()).unwrap().value, 42);
        assert!(!reg.contains(&"nonexistent".to_string()));
    }

    #[test]
    fn registry_keys_and_iter() {
        let mut reg = TestRegistry::default();
        reg.register_def(TestDef {
            id: "a".into(),
            value: 1,
        })
        .unwrap();
        reg.register_def(TestDef {
            id: "b".into(),
            value: 2,
        })
        .unwrap();

        let keys = reg.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&&"a".to_string()));
        assert!(keys.contains(&&"b".to_string()));

        let entries: Vec<_> = reg.iter().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn registry_rejects_duplicate_key() {
        let mut reg = TestRegistry::default();
        reg.register_def(TestDef {
            id: "dup".into(),
            value: 1,
        })
        .unwrap();
        let result = reg.register_def(TestDef {
            id: "dup".into(),
            value: 2,
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn registry_is_send_sync() {
        fn check_send_sync<T: Send + Sync>() {}
        check_send_sync::<TestRegistry>();
    }

    // ── RON loading tests ──

    #[test]
    fn loadable_registry_load_from_file() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("registry_test_load_file");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.ron");

        let ron_content = "[
            (id: \"item_1\", value: 10),
            (id: \"item_2\", value: 20),
        ]";
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(ron_content.as_bytes()).unwrap();

        let reg: TestRegistry = TestRegistry::load_from_file(&file_path).unwrap();
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get(&"item_1".to_string()).unwrap().value, 10);
        assert_eq!(reg.get(&"item_2".to_string()).unwrap().value, 20);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loadable_registry_load_from_dir() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("registry_test_load_dir");
        let _ = std::fs::create_dir_all(&dir);

        let mut file1 = std::fs::File::create(dir.join("a.ron")).unwrap();
        file1.write_all(b"(id: \"alpha\", value: 100)").unwrap();

        let mut file2 = std::fs::File::create(dir.join("b.ron")).unwrap();
        file2.write_all(b"(id: \"beta\", value: 200)").unwrap();

        let reg: TestRegistry = TestRegistry::load_from_dir(&dir).unwrap();
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get(&"alpha".to_string()).unwrap().value, 100);
        assert_eq!(reg.get(&"beta".to_string()).unwrap().value, 200);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loadable_registry_load_from_dir_ignores_non_ron() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("registry_test_ignore_non_ron");
        let _ = std::fs::create_dir_all(&dir);

        let mut file1 = std::fs::File::create(dir.join("data.ron")).unwrap();
        file1.write_all(b"(id: \"only\", value: 1)").unwrap();

        // 非 .ron 文件应被忽略
        let mut file2 = std::fs::File::create(dir.join("notes.txt")).unwrap();
        file2.write_all(b"hello").unwrap();

        // 隐藏文件应被忽略
        let mut file3 = std::fs::File::create(dir.join(".hidden.ron")).unwrap();
        file3.write_all(b"(id: \"hidden\", value: 99)").unwrap();

        let reg: TestRegistry = TestRegistry::load_from_dir(&dir).unwrap();
        assert_eq!(reg.len(), 1);
        assert!(reg.contains(&"only".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn loadable_registry_load_from_dir_nonexistent_returns_empty() {
        let reg: TestRegistry =
            TestRegistry::load_from_dir("/tmp/nonexistent_registry_path_xyz").unwrap();
        assert!(reg.is_empty());
    }

    // ── LoadableSingleRegistry tests ──

    #[derive(Default, Debug)]
    struct SingleTestRegistry {
        items: HashMap<String, TestData>,
    }

    impl Registry for SingleTestRegistry {
        type Key = String;
        type Data = TestData;

        fn len(&self) -> usize {
            self.items.len()
        }

        fn get(&self, key: &Self::Key) -> Option<&Self::Data> {
            self.items.get(key)
        }

        fn keys(&self) -> Vec<&Self::Key> {
            self.items.keys().collect()
        }

        fn iter(&self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Data)> + '_> {
            Box::new(self.items.iter())
        }
    }

    #[derive(Debug, Clone, Deserialize)]
    struct MultiItemDef {
        items: Vec<TestDef>,
    }

    impl LoadableSingleRegistry for SingleTestRegistry {
        type Def = MultiItemDef;
        type Error = TestError;

        fn register_def(&mut self, def: Self::Def) -> Result<(), Self::Error> {
            for item in def.items {
                if self.items.contains_key(&item.id) {
                    return Err(TestError(format!("Duplicate key: {}", item.id)));
                }
                self.items.insert(item.id, TestData { value: item.value });
            }
            Ok(())
        }
    }

    #[test]
    fn single_registry_load_from_file() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("single_registry_test");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("config.ron");

        let ron_content = "(
            items: [
                (id: \"x\", value: 7),
                (id: \"y\", value: 8),
            ],
        )";
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(ron_content.as_bytes()).unwrap();

        let reg: SingleTestRegistry = SingleTestRegistry::load_from_file(&file_path).unwrap();
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get(&"x".to_string()).unwrap().value, 7);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
