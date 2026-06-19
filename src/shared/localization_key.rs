//! LocalizationKey — 强类型本地化 Key 包装器
//!
//! 替代全局 `String` 类型，提供零额外开销的编译期类型安全。
//! 所有 Definition 中的 `name_key`/`desc_key`/`icon_key` 应使用此类型。
//!
//! 详见 `docs/04-data/infrastructure/localization_schema.md` §1

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

/// 强类型本地化 Key。
///
/// 格式: `<namespace>.<scope>.<id>.<suffix>`
/// 示例: `ability.abl_000042.name`, `core.yes`
///
/// # 设计原则
/// - 零开销抽象：运行时等同 `String`，仅编译期附加类型信息
/// - 支持 serde 序列化/反序列化（RON 配置文件兼容）
/// - `Deref<Target=str>` 使其在大多数场景可替代 `&str`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalizationKey(pub String);

impl LocalizationKey {
    /// 创建新的 LocalizationKey。
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// 返回底层字符串引用。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for LocalizationKey {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LocalizationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for LocalizationKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for LocalizationKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<LocalizationKey> for String {
    fn from(key: LocalizationKey) -> String {
        key.0
    }
}

impl PartialEq<str> for LocalizationKey {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for LocalizationKey {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Default for LocalizationKey {
    fn default() -> Self {
        Self(String::new())
    }
}
