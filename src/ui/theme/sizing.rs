//! UiSizing — 布局尺寸常量 Resource
//!
//! 存储 Combat HUD 的 Zone 最小尺寸、Widget 固定尺寸和画布边距。
//! 与 UiSpacing（间距令牌）互补：Spacing 负责间距和间隙，
//! Sizing 负责固定尺寸和最小尺寸约束。
//!
//! UiSizing 是主题无关的（暗色/亮色主题共用相同值）。
//! 在 UiPlugin 中注册。
//!
//! 详见 `docs/09-planning/ui-layout-system-plan.md` §3.3

use bevy::prelude::*;

/// 布局尺寸常量 — Combat HUD Zone 和 Widget 的固定尺寸。
///
/// 所有值以逻辑像素为单位。遵循 4px 基准单位（BASE_UNIT），
/// 所有值必须是 4 的倍数。
#[derive(Resource, Debug, Clone, Reflect)]
pub struct UiSizing {
    // ── Zone 最小尺寸 ──
    /// 顶部 Zone（Z1-Z3）最小高度（56px）
    pub zone_min_height_top: f32,
    /// Z3 Top-Right 最小宽度（160px）
    pub zone_min_width_top_right: f32,
    /// Z5 Bottom-Left 最小宽度（240px）
    pub zone_min_width_bottom_left: f32,
    /// Z5 Bottom-Left 最小高度（180px）
    pub zone_min_height_bottom_left: f32,
    /// Z7 Bottom-Right 最小宽度（200px）
    pub zone_min_width_bottom_right: f32,
    /// Z8 Bottom Bar 高度（48px）
    pub zone_height_bottom_bar: f32,

    // ── Widget 固定尺寸 ──
    /// CharacterCard 宽度（280px）
    pub character_card_width: f32,
    /// SkillPanel 宽度（240px）
    pub skill_panel_width: f32,
    /// EndTurnButton 宽度（140px）
    pub end_turn_button_width: f32,
    /// TurnOrderBar 高度（48px）
    pub turn_order_bar_height: f32,
    /// TurnIndicator 最小宽度（100px）
    pub turn_indicator_min_width: f32,
    /// PhaseText 最小宽度（120px）
    pub phase_text_min_width: f32,

    // ── 画布边距 ──
    /// 屏幕画布边缘内边距（8px，与 theme.spacing.sm 一致）
    pub canvas_padding: f32,
}

impl Default for UiSizing {
    fn default() -> Self {
        Self {
            // Zone 最小尺寸
            zone_min_height_top: 56.0,
            zone_min_width_top_right: 160.0,
            zone_min_width_bottom_left: 240.0,
            zone_min_height_bottom_left: 180.0,
            zone_min_width_bottom_right: 200.0,
            zone_height_bottom_bar: 48.0,

            // Widget 固定尺寸
            character_card_width: 280.0,
            skill_panel_width: 240.0,
            end_turn_button_width: 140.0,
            turn_order_bar_height: 48.0,
            turn_indicator_min_width: 100.0,
            phase_text_min_width: 120.0,

            // 画布边距
            canvas_padding: 8.0,
        }
    }
}
