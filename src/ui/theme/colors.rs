//! 语义颜色令牌定义
//!
//! Widget 必须引用语义令牌（如 `theme.colors.text_primary`）
//! 而非原始 RGB 值。这使得切换主题时无需修改 Widget 代码。
//!
//! 参见 `docs/06-ui/02-design-system/theme-localization.md` §2.3

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// UI 主题的语义颜色令牌。
///
/// 所有值在 sRGB 颜色空间中定义。Widget 不得使用
/// 原始 `Color::srgb(...)` 或 `Color::WHITE` / `Color::BLACK` —
/// 始终引用此结构体中的令牌。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct UiColors {
    // ── UI 表面颜色 ──
    /// 主要表面背景（面板、菜单）
    pub surface_primary: Color,
    /// 次要表面背景（子面板、工具提示）
    pub surface_secondary: Color,
    /// 危险/破坏性操作的表面
    pub surface_danger: Color,
    /// 禁用表面（非交互元素）
    pub surface_disabled: Color,
    /// 次要表面悬停状态
    pub surface_secondary_hover: Color,
    /// 次要表面按下状态
    pub surface_secondary_pressed: Color,
    /// 危险表面悬停状态
    pub surface_danger_hover: Color,
    /// 危险表面按下状态
    pub surface_danger_pressed: Color,

    // ── 文本颜色 ──
    /// 主要文本颜色（正文内容）
    pub text_primary: Color,
    /// 次要文本颜色（标签、说明）
    pub text_secondary: Color,
    /// 禁用文本颜色
    pub text_disabled: Color,
    /// 强调文本颜色（高亮、关键值）
    pub text_accent: Color,

    // ── 强调/交互颜色 ──
    /// 主要强调颜色（按钮、交互元素）
    pub accent_primary: Color,
    /// 悬停状态强调色
    pub accent_hover: Color,
    /// 按下状态强调色
    pub accent_pressed: Color,

    // ── Ghost 按钮颜色（透明背景变体） ──
    /// Ghost 按钮默认背景（透明）
    pub ghost: Color,
    /// Ghost 按钮悬停背景
    pub ghost_hover: Color,
    /// Ghost 按钮按下背景
    pub ghost_pressed: Color,

    // ── 反馈颜色 ──
    /// 正面反馈（治疗、成功）
    pub feedback_positive: Color,
    /// 负面反馈（伤害、错误）
    pub feedback_negative: Color,
    /// 警告反馈（注意、告警）
    pub feedback_warning: Color,

    // ── 边框颜色 ──
    /// 默认边框颜色
    pub border_default: Color,
    /// 聚焦/高亮边框颜色
    pub border_focus: Color,
}

impl UiColors {
    /// 暗色主题调色板。
    pub fn dark() -> Self {
        Self {
            surface_primary: Color::srgb(0.11, 0.11, 0.14),
            surface_secondary: Color::srgb(0.16, 0.16, 0.20),
            surface_danger: Color::srgb(0.24, 0.08, 0.08),
            surface_disabled: Color::srgb(0.18, 0.18, 0.20),
            surface_secondary_hover: Color::srgb(0.20, 0.20, 0.25),
            surface_secondary_pressed: Color::srgb(0.13, 0.13, 0.17),
            surface_danger_hover: Color::srgb(0.30, 0.10, 0.10),
            surface_danger_pressed: Color::srgb(0.18, 0.06, 0.06),
            text_primary: Color::srgb(0.90, 0.90, 0.92),
            text_secondary: Color::srgb(0.65, 0.65, 0.70),
            text_disabled: Color::srgb(0.40, 0.40, 0.45),
            text_accent: Color::srgb(0.94, 0.77, 0.25),
            accent_primary: Color::srgb(0.29, 0.56, 0.85),
            accent_hover: Color::srgb(0.36, 0.63, 0.91),
            accent_pressed: Color::srgb(0.23, 0.47, 0.76),
            ghost: Color::NONE,
            ghost_hover: Color::srgba(1.0, 1.0, 1.0, 0.10),
            ghost_pressed: Color::srgba(1.0, 1.0, 1.0, 0.18),
            feedback_positive: Color::srgb(0.30, 0.69, 0.31),
            feedback_negative: Color::srgb(0.82, 0.18, 0.18),
            feedback_warning: Color::srgb(0.95, 0.61, 0.07),
            border_default: Color::srgb(0.33, 0.33, 0.36),
            border_focus: Color::srgb(0.29, 0.56, 0.85),
        }
    }

    /// 亮色主题调色板。
    pub fn light() -> Self {
        Self {
            surface_primary: Color::srgb(0.97, 0.97, 0.98),
            surface_secondary: Color::srgb(0.92, 0.92, 0.94),
            surface_danger: Color::srgb(0.98, 0.85, 0.85),
            surface_disabled: Color::srgb(0.88, 0.88, 0.90),
            surface_secondary_hover: Color::srgb(0.88, 0.88, 0.91),
            surface_secondary_pressed: Color::srgb(0.95, 0.95, 0.96),
            surface_danger_hover: Color::srgb(0.97, 0.78, 0.78),
            surface_danger_pressed: Color::srgb(0.99, 0.90, 0.90),
            text_primary: Color::srgb(0.13, 0.13, 0.16),
            text_secondary: Color::srgb(0.45, 0.45, 0.50),
            text_disabled: Color::srgb(0.65, 0.65, 0.70),
            text_accent: Color::srgb(0.71, 0.53, 0.04),
            accent_primary: Color::srgb(0.20, 0.46, 0.78),
            accent_hover: Color::srgb(0.27, 0.53, 0.85),
            accent_pressed: Color::srgb(0.14, 0.38, 0.69),
            ghost: Color::NONE,
            ghost_hover: Color::srgba(0.0, 0.0, 0.0, 0.06),
            ghost_pressed: Color::srgba(0.0, 0.0, 0.0, 0.12),
            feedback_positive: Color::srgb(0.18, 0.55, 0.20),
            feedback_negative: Color::srgb(0.72, 0.08, 0.08),
            feedback_warning: Color::srgb(0.85, 0.51, 0.02),
            border_default: Color::srgb(0.70, 0.70, 0.73),
            border_focus: Color::srgb(0.20, 0.46, 0.78),
        }
    }
}

impl Default for UiColors {
    fn default() -> Self {
        Self::dark()
    }
}
