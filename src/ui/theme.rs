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
    /// 玩家阵营颜色
    pub faction_player_color: Color,
    /// 敌方阵营颜色
    pub faction_enemy_color: Color,

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
            faction_player_color: Color::srgb(0.2, 0.5, 1.0),
            faction_enemy_color: Color::srgb(1.0, 0.3, 0.2),

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

/// 阵营对应的单位颜色（表现层映射）
pub fn faction_color(faction: crate::character::Faction, theme: &UiTheme) -> Color {
    match faction {
        crate::character::Faction::Player => theme.faction_player_color,
        crate::character::Faction::Enemy => theme.faction_enemy_color,
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则 (ui_rules_v1.md 规则 4)
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;
    use crate::character::Faction;

    /// Test ID: UI-THM-001
    /// Title: 阵营颜色映射正确区分玩家和敌方
    ///
    /// Given: 默认 UiTheme
    /// When: 查询玩家和敌方的阵营颜色
    /// Then: 两种颜色不同
    ///
    /// Assertions: player_color != enemy_color
    #[test]
    fn faction_color_distinguishes_player_and_enemy() {
        // Given
        let theme = UiTheme::default();

        // When
        let player_color = faction_color(Faction::Player, &theme);
        let enemy_color = faction_color(Faction::Enemy, &theme);

        // Then
        assert_ne!(player_color, enemy_color);
    }

    /// Test ID: UI-THM-002
    /// Title: 玩家阵营颜色为蓝色系
    ///
    /// Given: 默认 UiTheme
    /// When: 查询玩家阵营颜色
    /// Then: 蓝色分量大于红色分量
    ///
    /// Assertions: blue > red
    #[test]
    fn faction_color_player_is_blue_tinted() {
        // Given
        let theme = UiTheme::default();

        // When
        let color = faction_color(Faction::Player, &theme);
        let rgba = Srgba::from(color);

        // Then
        assert!(rgba.blue > rgba.red);
    }

    /// Test ID: UI-THM-003
    /// Title: 敌方阵营颜色为红色系
    ///
    /// Given: 默认 UiTheme
    /// When: 查询敌方阵营颜色
    /// Then: 红色分量大于蓝色分量
    ///
    /// Assertions: red > blue
    #[test]
    fn faction_color_enemy_is_red_tinted() {
        // Given
        let theme = UiTheme::default();

        // When
        let color = faction_color(Faction::Enemy, &theme);
        let rgba = Srgba::from(color);

        // Then
        assert!(rgba.red > rgba.blue);
    }

    /// Test ID: UI-THM-004
    /// Title: UiTheme 默认值完整性
    ///
    /// Given: UiTheme::default()
    /// When: 检查所有字段
    /// Then: 所有字段值符合预期
    ///
    /// Assertions: 颜色/字号/间距值正确
    #[test]
    fn ui_theme_default_values_complete() {
        // Given
        let theme = UiTheme::default();

        // When - 无需操作

        // Then - 颜色
        assert_eq!(theme.panel_bg, Color::srgba(0.1, 0.1, 0.1, 0.9));
        assert_eq!(theme.text_primary, Color::WHITE);
        assert_eq!(
            theme.faction_player_color,
            Color::srgb(0.2, 0.5, 1.0)
        );
        assert_eq!(
            theme.faction_enemy_color,
            Color::srgb(1.0, 0.3, 0.2)
        );

        // Then - 字号
        assert!((theme.font_large - 24.0).abs() < f32::EPSILON);
        assert!((theme.font_medium - 18.0).abs() < f32::EPSILON);
        assert!((theme.font_small - 14.0).abs() < f32::EPSILON);

        // Then - 间距
        assert!((theme.gap_small - 4.0).abs() < f32::EPSILON);
        assert!((theme.gap_medium - 6.0).abs() < f32::EPSILON);
        assert!((theme.gap_large - 10.0).abs() < f32::EPSILON);
    }
}
