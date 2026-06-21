//! BCP-47 语言标识符枚举
//!
//! 用类型安全的枚举替代之前的 `pub type LocaleId = String;`。
//! 实现 serde 重命名以兼容配置文件。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// BCP-47 语言标识符。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum LocaleId {
    /// 美式英语
    #[serde(rename = "en-US")]
    EnUS,
    /// 简体中文
    #[serde(rename = "zh-CN")]
    ZhCN,
    /// 日语
    #[serde(rename = "ja-JP")]
    JaJP,
    /// 用于测试/QA 的伪区域设置
    #[serde(rename = "zz-ZZ")]
    ZzZZ,
}

impl LocaleId {
    /// 返回 BCP-47 字符串表示。
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
