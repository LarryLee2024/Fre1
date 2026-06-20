//! LocaleId — BCP-47 language identifier enum
//!
//! Replaces the former `pub type LocaleId = String;` with a type-safe enum.
//! Implements serde renaming for config file compatibility.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// BCP-47 language identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum LocaleId {
    /// American English
    #[serde(rename = "en-US")]
    EnUS,
    /// Simplified Chinese
    #[serde(rename = "zh-CN")]
    ZhCN,
    /// Japanese
    #[serde(rename = "ja-JP")]
    JaJP,
    /// Fake locale for testing / QA
    #[serde(rename = "zz-ZZ")]
    ZzZZ,
}

impl LocaleId {
    /// Return the BCP-47 string representation.
    pub fn as_str(&self) -> &str {
        match self {
            LocaleId::EnUS => "en-US",
            LocaleId::ZhCN => "zh-CN",
            LocaleId::JaJP => "ja-JP",
            LocaleId::ZzZZ => "zz-ZZ",
        }
    }
}

impl AsRef<str> for LocaleId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for LocaleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for LocaleId {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "en-US" => Ok(Self::EnUS),
            "zh-CN" => Ok(Self::ZhCN),
            "ja-JP" => Ok(Self::JaJP),
            "zz-ZZ" => Ok(Self::ZzZZ),
            other => Err(format!("Unknown locale identifier: {}", other)),
        }
    }
}
