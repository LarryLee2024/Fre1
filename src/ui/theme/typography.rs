//! 字体和文本样式令牌
//!
//! Widget 必须引用字体令牌（如 `theme.typography.size_body`）
//! 而非原始字体大小或硬编码字体路径。这使得文本样式一致
//! 并支持全局字体更改。
//!
//! 参见 `docs/06-ui/02-design-system/theme-localization.md` §2.5

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// UI 主题的字体令牌。
///
/// 字体路径引用 `assets/fonts/` 目录中的资源。
/// Widget 不得硬编码字体大小或字体路径 —
/// 始终引用此结构体中的令牌。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct UiTypography {
    // ── 字体路径 ──
    /// 正文字体资源路径
    pub font_body: String,
    /// 标题字体资源路径
    pub font_heading: String,
    /// 等宽字体资源路径（用于数字、代码）
    pub font_mono: String,

    // ── 字体大小 ──
    /// 正文字体大小（14px）
    pub size_body: f32,
    /// 小号/说明文字大小（12px）
    pub size_small: f32,
    /// 标题字体大小（18px）
    pub size_heading: f32,
    /// 大标题字体大小（24px）
    pub size_title: f32,
    /// 展示/超大标题大小（36px）
    pub size_display: f32,

    // ── 字体粗细 ──
    /// 常规字体粗细
    pub weight_normal: f32,
    /// 粗体字体粗细
    pub weight_bold: f32,
}

impl UiTypography {
    /// 默认字体值（跨主题共享）。
    pub fn default_values() -> Self {
        Self {
            font_body: "fonts/FiraSans-Regular.ttf".into(),
            font_heading: "fonts/FiraSans-Bold.ttf".into(),
            font_mono: "fonts/FiraCode-Regular.ttf".into(),
            size_body: 14.0,
            size_small: 12.0,
            size_heading: 18.0,
            size_title: 24.0,
            size_display: 36.0,
            weight_normal: 400.0,
            weight_bold: 700.0,
        }
    }
}

impl Default for UiTypography {
    fn default() -> Self {
        Self::default_values()
    }
}
