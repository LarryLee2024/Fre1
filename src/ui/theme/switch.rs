//! Theme switching — 运行时主题切换

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Theme;

/// 主题变体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeVariant {
    /// 返回主题变体的字符串标识（"dark" 或 "light"）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 切换主题的纯函数
pub fn switch_theme(theme: &mut Theme, variant: ThemeVariant) {
    theme.colors = match variant {
        ThemeVariant::Dark => super::UiColors::dark(),
        ThemeVariant::Light => super::UiColors::light(),
    };
}
