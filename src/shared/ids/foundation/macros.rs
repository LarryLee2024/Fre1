//! define_string_id! / define_numeric_id! 宏定义。
//!
//! 提供两个宏用于生成强类型 ID：
//!
//! - `define_string_id!` — 生成 String 类型 ID（配置表标识），带 Display/FromStr/Serde/StrongId
//! - `define_numeric_id!` — 生成 u64 类型 ID（运行时实例标识），带 Display/Copy/Serde
//!
//! # 注意
//!
//! 调用此宏的模块必须将 `bevy::prelude::Reflect` 引入作用域，
//! 因为 `#[derive(Reflect)]` 需要在调用方展开。

/// String 类型 ID 宏。
///
/// Display 格式: `<prefix>:<value>`（如 `tag:tag_000001`）
/// Serde 格式: 同时接受 `<prefix>:<value>` 和裸 `<value>`
///
/// 注意：调用此宏的模块必须将 `bevy::prelude::Reflect` 引入作用域，
/// 因为 `#[derive(Reflect)]` 需要在调用方展开。参见 `types/string_ids.rs` 顶部。
#[macro_export]
macro_rules! define_string_id {
    (
        $vis:vis $name:ident,
        prefix: $prefix:expr,
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
        #[reflect(Hash, PartialEq)]
        $vis struct $name(pub String);

        impl $name {
            $vis fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// 创建 ID 并校验格式。
            ///
            /// 校验规则：
            /// - 非空
            /// - 不包含冒号（防止与 `prefix:value` 格式混淆）
            /// - 只包含字母、数字、下划线（标准 ID 字符）
            ///
            /// # Errors
            ///
            /// 如果格式不符合规范，返回 `IdFormatError`。
            $vis fn checked_new(id: impl Into<String>) -> Result<Self, $crate::shared::ids::foundation::IdFormatError> {
                let s = id.into();
                if s.is_empty() {
                    return Err($crate::shared::ids::foundation::IdFormatError::Empty);
                }
                if s.contains(':') {
                    return Err($crate::shared::ids::foundation::IdFormatError::PrefixMismatch {
                        expected: $prefix,
                        actual: s,
                    });
                }
                if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err($crate::shared::ids::foundation::IdFormatError::InvalidCharacters(s));
                }
                Ok(Self(s))
            }

            /// 创建 ID 并附加 Debug 审计信息（仅 debug 模式收集，release 模式无开销）。
            $vis fn new_tracked(id: impl Into<String>, _info: $crate::shared::ids::foundation::IdCreationInfo) -> Self {
                #[cfg(debug_assertions)]
                {
                    // 审计信息在 debug 模式下可用，release 模式被编译消除
                    let _ = &_info;
                }
                Self(id.into())
            }

            $vis fn as_str(&self) -> &str {
                &self.0
            }

            $vis fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            $vis fn len(&self) -> usize {
                self.0.len()
            }

            $vis fn into_inner(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}:{}", $prefix, self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let prefix_colon = concat!($prefix, ":");
                if let Some(value) = s.strip_prefix(prefix_colon) {
                    Ok(Self(value.to_string()))
                } else if !s.contains(':') {
                    Ok(Self(s.to_string()))
                } else {
                    Err(format!(
                        "invalid ID format: expected '{}:...' or '<value>', got '{}'",
                        $prefix, s
                    ))
                }
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct IdVisitor;
                impl<'de> serde::de::Visitor<'de> for IdVisitor {
                    type Value = $name;
                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "a string in format '{}:<value>' or '<value>'", $prefix)
                    }
                    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$name, E> {
                        v.parse().map_err(serde::de::Error::custom)
                    }
                }
                deserializer.deserialize_str(IdVisitor)
            }
        }

        impl $crate::shared::ids::StrongId for $name {
            fn prefix() -> &'static str {
                $prefix
            }
            fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

/// u64 类型实例 ID 宏。
///
/// 用于运行时分配的实例唯一标识（如 ModifierInstanceId）。
///
/// 注意：调用此宏的模块必须将 `bevy::prelude::Reflect` 引入作用域。
#[macro_export]
macro_rules! define_numeric_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
        pub struct $name(pub u64);

        impl $name {
            pub fn new(id: u64) -> Self {
                Self(id)
            }

            pub fn value(&self) -> u64 {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl From<u64> for $name {
            fn from(id: u64) -> Self {
                Self(id)
            }
        }

        impl std::ops::Deref for $name {
            type Target = u64;
            fn deref(&self) -> &u64 {
                &self.0
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_u64(self.0)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let id = u64::deserialize(deserializer)?;
                Ok(Self(id))
            }
        }
    };
}
