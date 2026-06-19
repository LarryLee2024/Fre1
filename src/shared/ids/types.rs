//! 领域 ID 类型（由宏统一生成）

use bevy::prelude::Reflect;

/// String 类型 ID 宏。
///
/// Display 格式: `<prefix>:<value>`（如 `tag:tag_000001`）
/// Serde 格式: 同时接受 `<prefix>:<value>` 和裸 `<value>`
///
/// 注意：调用此宏的模块必须将 `bevy::prelude::Reflect` 引入作用域，
/// 因为 `#[derive(Reflect)]` 需要在调用方展开。参见 `types.rs` 顶部。
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
#[macro_export]
macro_rules! define_numeric_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

// ============================================================================
// String ID 类型（领域 Definition 标识）
// ============================================================================

define_string_id! {
    pub AttributeId,
    prefix: "attr",
}

define_string_id! {
    pub TagId,
    prefix: "tag",
}

define_string_id! {
    pub ModifierId,
    prefix: "mod",
}

define_string_id! {
    pub EffectId,
    prefix: "eff",
}

define_string_id! {
    pub AbilityId,
    prefix: "abl",
}

define_string_id! {
    pub TriggerId,
    prefix: "trg",
}

define_string_id! {
    pub CueId,
    prefix: "cue",
}

define_string_id! {
    pub CharacterId,
    prefix: "char",
}

define_string_id! {
    pub UnitId,
    prefix: "unit",
}

define_string_id! {
    pub EquipmentId,
    prefix: "equip",
}

define_string_id! {
    pub ItemId,
    prefix: "itm",
}

define_string_id! {
    pub FactionId,
    prefix: "fct",
}

// ============================================================================
// 补充领域 ID 类型（按 id_strategy.md table 新增）
// ============================================================================

define_string_id! {
    pub QuestId,
    prefix: "qst",
}

define_string_id! {
    pub SpellId,
    prefix: "spl",
}

define_string_id! {
    pub BuffId,
    prefix: "buf",
}

define_string_id! {
    pub TerrainId,
    prefix: "ter",
}

define_string_id! {
    pub RecipeId,
    prefix: "rcp",
}

define_string_id! {
    pub LootTableId,
    prefix: "ltb",
}

define_string_id! {
    pub TeamId,
    prefix: "team",
}

define_string_id! {
    pub ClassId,
    prefix: "cls",
}

define_string_id! {
    pub TalentId,
    prefix: "tal",
}

define_string_id! {
    pub SubclassId,
    prefix: "sub",
}

define_string_id! {
    pub BondDefId,
    prefix: "bnd",
}

define_string_id! {
    pub FormationDefId,
    prefix: "fmd",
}

define_string_id! {
    pub CampEventId,
    prefix: "cmp",
}

// ============================================================================
// DefinitionId — 通用 Definition ID（无前缀，用于 Registry 系统）
// ============================================================================

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
