//! 设计间距比例令牌
//!
//! Widget 必须引用语义间距令牌（如 `theme.spacing.md`）
//! 而非原始 `Val::Px(16.0)` 值。这确保整个 UI 间距一致
//! 并支持全局间距调整。
//!
//! 参见 `docs/06-ui/02-design-system/theme-localization.md` §2.4

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// UI 主题的设计间距比例。
///
/// 所有值以逻辑像素为单位。Widget 不得硬编码
/// `Val::Px(...)` — 始终引用此结构体中的令牌。
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct UiSpacing {
    // ── 间距比例 ──
    /// 超小间距（4px）
    pub xs: f32,
    /// 小间距（8px）
    pub sm: f32,
    /// 中间距（16px）
    pub md: f32,
    /// 大间距（24px）
    pub lg: f32,
    /// 超大间距（32px）
    pub xl: f32,
    /// 双超大间距（48px）
    pub xxl: f32,

    // ── 特定尺寸 ──
    /// 小圆角（4px）
    pub border_radius_sm: f32,
    /// 中圆角（8px）
    pub border_radius_md: f32,
    /// 大圆角（12px）
    pub border_radius_lg: f32,
    /// 标准图标大小（32px）
    pub icon_size: f32,
    /// 标准按钮高度（40px）
    pub button_height: f32,
    /// 最小触摸目标大小（44px，无障碍）
    pub min_touch_target: f32,
}

impl UiSpacing {
    /// 默认间距值（暗色和亮色主题共享相同比例）。
    pub fn default_scale() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            border_radius_sm: 4.0,
            border_radius_md: 8.0,
            border_radius_lg: 12.0,
            icon_size: 32.0,
            button_height: 40.0,
            min_touch_target: 44.0,
        }
    }
}

impl Default for UiSpacing {
    fn default() -> Self {
        Self::default_scale()
    }
}
