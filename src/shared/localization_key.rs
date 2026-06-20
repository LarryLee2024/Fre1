//! LocalizationKey — 强类型本地化 Key 包装器
//!
//! 替代全局 `String` 类型，提供零额外开销的编译期类型安全。
//! 所有 Definition 中的 `name_key`/`desc_key`/`icon_key` 应使用此类型。
//!
//! 内部使用 `Cow<'static, str>` 实现零拷贝：
//! - `Cow::Borrowed` 用于编译期已知的静态 Key 常量（零分配），通过 `from_static()` 创建
//! - `Cow::Owned` 用于运行时构造的 Key，通过 `new()` 创建
//!
//! 详见 `docs/04-data/infrastructure/localization_schema.md` §1

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;

/// 强类型本地化 Key。
///
/// 格式: `<namespace>.<scope>.<id>.<suffix>`
/// 示例: `ability.abl_000042.name`, `core.yes`
///
/// # 设计原则
/// - 零开销抽象：运行时等同 `Cow<'static, str>`，编译期 Key 零分配
/// - 支持 serde 序列化/反序列化（RON 配置文件兼容）
/// - `Deref<Target=str>` 使其在大多数场景可替代 `&str`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[derive(Default)]
pub struct LocalizationKey(pub Cow<'static, str>);

impl LocalizationKey {
    /// 创建新的 LocalizationKey（运行时构造，Cow::Owned）。
    ///
    /// 接受 `String` 或 `&str`，内部始终以 Cow::Owned 存储。
    /// 对于编译期已知的静态 Key 常量，使用 `from_static()` 实现零分配。
    pub fn new(key: impl Into<String>) -> Self {
        Self(Cow::Owned(key.into()))
    }

    /// 从静态字符串常量创建 LocalizationKey（零分配，Cow::Borrowed）。
    ///
    /// 用于编译期已知的 Key 常量，如 `generated/keys.rs` 中的 `&'static str` 常量。
    /// 运行时零开销——不产生堆分配。
    pub const fn from_static(key: &'static str) -> Self {
        Self(Cow::Borrowed(key))
    }

    /// 返回底层字符串引用。
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 返回内部 Cow 引用。
    pub fn as_cow(&self) -> &Cow<'static, str> {
        &self.0
    }

    /// 消耗 self，返回拥有的 String（必要时克隆）。
    pub fn into_string(self) -> String {
        self.0.into_owned()
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
        Self(Cow::Owned(s))
    }
}

impl From<&str> for LocalizationKey {
    fn from(s: &str) -> Self {
        Self(Cow::Owned(s.to_string()))
    }
}

impl From<LocalizationKey> for String {
    fn from(key: LocalizationKey) -> String {
        key.0.into_owned()
    }
}

impl PartialEq<str> for LocalizationKey {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl PartialEq<&str> for LocalizationKey {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref() == *other
    }
}
