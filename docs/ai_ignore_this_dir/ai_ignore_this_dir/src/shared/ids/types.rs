//! 全部 22 个领域 ID 类型（由 `define_id!` 宏统一生成）

macro_rules! define_id {
    (
        $vis:vis $name:ident,
        prefix: $prefix:expr,
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $vis struct $name(pub String);

        impl $name {
            #[allow(dead_code)]
            $vis fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            #[allow(dead_code)]
            $vis fn as_str(&self) -> &str {
                &self.0
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
                let expected = $prefix;
                let prefix_colon = format!("{}:", expected);
                if let Some(value) = s.strip_prefix(&prefix_colon) {
                    Ok(Self(value.to_string()))
                } else {
                    Err(format!(
                        "Invalid ID format: expected '{}:...', got '{}'",
                        expected, s
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
                        write!(f, "a string in format '{}:<value>'", $prefix)
                    }
                    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<$name, E> {
                        v.parse().map_err(serde::de::Error::custom)
                    }
                }
                deserializer.deserialize_str(IdVisitor)
            }
        }

        impl crate::shared::ids::StrongId for $name {
            fn prefix() -> &'static str {
                $prefix
            }
            fn as_str(&self) -> &str {
                &self.0
            }
        }

    };
}

// ============================================================================
// 22 个领域 ID 类型
// ============================================================================

define_id! {
    pub AttributeId,
    prefix: "attr",
}

define_id! {
    pub TagId,
    prefix: "tag",
}

define_id! {
    pub ModifierId,
    prefix: "mod",
}

define_id! {
    pub EffectId,
    prefix: "eff",
}

define_id! {
    pub StackingId,
    prefix: "stack",
}

define_id! {
    pub ExecutionId,
    prefix: "exec",
}

define_id! {
    pub AbilityId,
    prefix: "ability",
}

define_id! {
    pub TriggerId,
    prefix: "trig",
}

define_id! {
    pub TargetingId,
    prefix: "tgt",
}

define_id! {
    pub CueId,
    prefix: "cue",
}

define_id! {
    pub CharacterId,
    prefix: "char",
}

define_id! {
    pub UnitId,
    prefix: "unit",
}

define_id! {
    pub EquipmentId,
    prefix: "equip",
}

define_id! {
    pub ItemId,
    prefix: "item",
}

define_id! {
    pub TerrainId,
    prefix: "terrain",
}

define_id! {
    pub ClassId,
    prefix: "class",
}

define_id! {
    pub RaceId,
    prefix: "race",
}

define_id! {
    pub TraitId,
    prefix: "trait",
}

define_id! {
    pub AiBehaviorId,
    prefix: "ai",
}

define_id! {
    pub CampaignId,
    prefix: "camp",
}

define_id! {
    pub StageId,
    prefix: "stage",
}

define_id! {
    pub FactionId,
    prefix: "faction",
}
