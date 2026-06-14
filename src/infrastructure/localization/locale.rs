//! 语言标识类型

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 支持的语言枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "ja-JP")]
    JaJp,
    #[serde(rename = "ko-KR")]
    KoKr,
}

impl Default for Locale {
    fn default() -> Self {
        Self::ZhCn
    }
}

impl Locale {
    /// 返回语言标识字符串（BCP 47 格式）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::EnUs => "en-US",
            Self::JaJp => "ja-JP",
            Self::KoKr => "ko-KR",
        }
    }

    /// 返回语言显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ZhCn => "简体中文",
            Self::EnUs => "English",
            Self::JaJp => "日本語",
            Self::KoKr => "한국어",
        }
    }

    /// 从字符串解析语言标识
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "zh-cn" | "zh" | "cn" => Self::ZhCn,
            "en-us" | "en" => Self::EnUs,
            "ja-jp" | "ja" => Self::JaJp,
            "ko-kr" | "ko" => Self::KoKr,
            _ => Self::default(),
        }
    }

    /// 所有支持的语言
    pub fn all() -> &'static [Locale] {
        &[Self::ZhCn, Self::EnUs, Self::JaJp, Self::KoKr]
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 当前语言 Resource
#[derive(Resource, Debug, Clone)]
pub struct CurrentLocale(pub Locale);

impl Default for CurrentLocale {
    fn default() -> Self {
        Self(Locale::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_default_is_zh_cn() {
        assert_eq!(Locale::default(), Locale::ZhCn);
    }

    #[test]
    fn locale_as_str() {
        assert_eq!(Locale::ZhCn.as_str(), "zh-CN");
        assert_eq!(Locale::EnUs.as_str(), "en-US");
        assert_eq!(Locale::JaJp.as_str(), "ja-JP");
        assert_eq!(Locale::KoKr.as_str(), "ko-KR");
    }

    #[test]
    fn locale_display_name() {
        assert_eq!(Locale::ZhCn.display_name(), "简体中文");
        assert_eq!(Locale::EnUs.display_name(), "English");
    }

    #[test]
    fn locale_from_str() {
        assert_eq!(Locale::from_str_loose("zh-CN"), Locale::ZhCn);
        assert_eq!(Locale::from_str_loose("en"), Locale::EnUs);
        assert_eq!(Locale::from_str_loose("ja-jp"), Locale::JaJp);
        assert_eq!(Locale::from_str_loose("unknown"), Locale::ZhCn);
    }

    #[test]
    fn locale_all_count() {
        assert_eq!(Locale::all().len(), 4);
    }
}
