// 主题系统：统一样式常量，避免颜色/字号/间距散落各文件
// 换皮肤只需修改 Theme，UI 代码不用动

use bevy::prelude::*;

/// UI 主题
#[derive(Resource, Debug, Clone)]
pub struct UiTheme {
    // ── 颜色 ──
    /// 面板背景色
    pub panel_bg: Color,
    /// 按钮背景色
    pub button_bg: Color,
    /// 按钮悬停色
    pub button_hover: Color,
    /// 主文本色
    pub text_primary: Color,
    /// 次文本色
    pub text_secondary: Color,
    /// 技能文本色
    pub text_skill: Color,
    /// 取消文本色
    pub text_cancel: Color,
    /// 伤害数字色
    pub damage_color: Color,
    /// 暴击数字色
    pub crit_color: Color,
    /// 治疗数字色
    pub heal_color: Color,
    /// 移动范围色
    pub movable_range: Color,
    /// 攻击范围色
    pub attack_range: Color,
    /// 选中高亮色
    pub selection_highlight: Color,
    /// 地形浮窗背景色
    pub tile_info_bg: Color,
    /// 地形浮窗文本色
    pub tile_info_text: Color,
    /// HP 进度条颜色
    pub hp_bar_color: Color,
    /// MP 进度条颜色
    pub mp_bar_color: Color,
    /// 耐力进度条颜色
    pub stamina_bar_color: Color,
    /// 增益 Buff 颜色
    pub buff_color: Color,
    /// 减益 Debuff 颜色
    pub debuff_color: Color,
    /// 进度条背景色
    pub bar_bg: Color,
    /// 分隔线颜色
    pub divider_color: Color,

    // ── 字号 ──
    /// 大标题字号
    pub font_large: f32,
    /// 正文字号
    pub font_medium: f32,
    /// 小字号
    pub font_small: f32,
    /// 菜单按钮字号
    pub font_menu: f32,
    /// 日志字号
    pub font_log: f32,
    /// 伤害数字字号
    pub font_damage: f32,
    /// 暴击数字字号
    pub font_crit: f32,
    /// 行动顺序标签字号
    pub font_turn_order: f32,

    // ── 间距 ──
    /// 按钮内边距
    pub button_padding: UiRect,
    /// 面板内边距
    pub panel_padding: UiRect,
    /// 小间距
    pub gap_small: f32,
    /// 中间距
    pub gap_medium: f32,
    /// 大间距
    pub gap_large: f32,

    // ── 布局常量 ──
    /// 单位信息面板宽度
    pub unit_panel_width: f32,
    /// 战斗日志面板宽度
    pub log_panel_width: f32,
    /// 战斗日志面板高度
    pub log_panel_height: f32,
    /// 资源条宽度
    pub bar_width: f32,
    /// 资源条高度
    pub bar_height: f32,
    /// 资源标签宽度
    pub bar_label_width: f32,
    /// 浮窗偏移量 (x, y)
    pub popup_offset: (f32, f32),
    /// 浮窗内边距
    pub popup_padding: UiRect,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            // 颜色
            panel_bg: Color::srgba(0.1, 0.1, 0.1, 0.9),
            button_bg: Color::NONE,
            button_hover: Color::srgba(0.3, 0.3, 0.3, 0.5),
            text_primary: Color::WHITE,
            text_secondary: Color::srgb(0.7, 0.7, 0.7),
            text_skill: Color::srgb(1.0, 0.8, 0.3),
            text_cancel: Color::srgb(0.7, 0.7, 0.7),
            damage_color: Color::srgb(1.0, 0.9, 0.3),
            crit_color: Color::srgb(1.0, 0.2, 0.2),
            heal_color: Color::srgb(0.3, 1.0, 0.3),
            movable_range: Color::srgba(0.3, 0.6, 1.0, 0.4),
            attack_range: Color::srgba(1.0, 0.3, 0.2, 0.35),
            selection_highlight: Color::srgba(1.0, 1.0, 0.3, 0.5),
            tile_info_bg: Color::srgba(0.05, 0.05, 0.1, 0.92),
            tile_info_text: Color::srgb(0.9, 0.9, 0.95),
            hp_bar_color: Color::srgb(0.9, 0.2, 0.2),
            mp_bar_color: Color::srgb(0.2, 0.4, 0.9),
            stamina_bar_color: Color::srgb(0.2, 0.8, 0.3),
            buff_color: Color::srgb(0.3, 1.0, 0.3),
            debuff_color: Color::srgb(1.0, 0.3, 0.3),
            bar_bg: Color::srgba(0.2, 0.2, 0.2, 0.8),
            divider_color: Color::srgb(0.4, 0.4, 0.4),

            // 字号
            font_large: 24.0,
            font_medium: 18.0,
            font_small: 14.0,
            font_menu: 16.0,
            font_log: 13.0,
            font_damage: 16.0,
            font_crit: 22.0,
            font_turn_order: 10.0,

            // 间距
            button_padding: UiRect::px(8.0, 8.0, 4.0, 4.0),
            panel_padding: UiRect::all(Val::Px(4.0)),
            gap_small: 4.0,
            gap_medium: 6.0,
            gap_large: 10.0,

            // 布局常量
            unit_panel_width: 380.0,
            log_panel_width: 420.0,
            log_panel_height: 280.0,
            bar_width: 150.0,
            bar_height: 10.0,
            bar_label_width: 28.0,
            popup_offset: (20.0, -40.0),
            popup_padding: UiRect::all(Val::Px(8.0)),
        }
    }
}
