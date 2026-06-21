//! 全局 UI 主题资源
//!
//! `Theme` 结构体是全局 ECS Resource，包含所有样式令牌
//! （颜色、间距、字体）。它在 UI 插件链中首先注册，
//! 以便所有 Widget 和界面可以访问主题值。
//!
//! 暗色主题为默认。亮色主题作为变体提供，
//! 用于未来的主题切换支持。
//!
//! 参见 `docs/06-ui/02-design-system/theme-localization.md` §2

use bevy::prelude::*;

use super::colors::UiColors;
use super::spacing::UiSpacing;
use super::typography::UiTypography;

/// 全局 UI 主题资源。
///
/// 由 `ThemePlugin` 插入到 ECS 中。所有 UI Widget 通过此资源访问样式
/// 令牌 — 不通过硬编码值。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Theme {
    /// 主题名称标识符（如 "dark"、"light"）
    pub name: &'static str,
    /// 语义颜色令牌
    pub colors: UiColors,
    /// 间距比例令牌
    pub spacing: UiSpacing,
    /// 字体令牌
    pub typography: UiTypography,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// 创建暗色主题变体。
    pub fn dark() -> Self {
        Self {
            name: "dark",
            colors: UiColors::dark(),
            spacing: UiSpacing::default_scale(),
            typography: UiTypography::default_values(),
        }
    }

    /// 创建亮色主题变体。
    pub fn light() -> Self {
        Self {
            name: "light",
            colors: UiColors::light(),
            spacing: UiSpacing::default_scale(),
            typography: UiTypography::default_values(),
        }
    }
}
