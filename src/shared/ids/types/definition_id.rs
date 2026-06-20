//! DefinitionId — 通用 Definition 标识符。
//!
//! 用于 Registry 系统中的泛型 Def 查询，不绑定特定前缀格式。
//! 实现 `StrongId` trait 以统一 ID 类型体系。

use bevy::prelude::Reflect;

use crate::shared::ids::StrongId;

/// 通用 Definition 标识符。
///
/// 用于 Registry 系统中的泛型 Def 查询，不绑定特定前缀格式。
/// 与 `define_string_id!` 生成的 ID 不同，DefinitionId 不要求前缀格式，
/// 可直接使用任意字符串作为 ID。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Hash, PartialEq)]
pub struct DefinitionId(pub String);

impl DefinitionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for DefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for DefinitionId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl std::ops::Deref for DefinitionId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for DefinitionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for DefinitionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DefinitionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl StrongId for DefinitionId {
    fn prefix() -> &'static str {
        "def"
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}
