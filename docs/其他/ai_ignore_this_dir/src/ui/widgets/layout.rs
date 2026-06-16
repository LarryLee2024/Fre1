// 布局 Widget：VBox/HBox/Panel/Label 等可复用布局辅助
// 对应 Godot 的 VBoxContainer/HBoxContainer/PanelContainer

use crate::ui::theme::UiTheme;
use bevy::prelude::*;

/// 纵向布局（相当于 Godot VBoxContainer）
#[allow(dead_code)]
pub fn vbox(theme: &UiTheme) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.gap_small),
        ..default()
    }
}

/// 纵向布局（自定义间距）
#[allow(dead_code)]
pub fn vbox_with_gap(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(gap),
        ..default()
    }
}

/// 横向布局（相当于 Godot HBoxContainer）
pub fn hbox(theme: &UiTheme) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(theme.gap_small),
        align_items: AlignItems::Center,
        ..default()
    }
}

/// 横向布局（自定义间距）
#[allow(dead_code)]
pub fn hbox_with_gap(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(gap),
        align_items: AlignItems::Center,
        ..default()
    }
}

/// 面板（相当于 Godot PanelContainer）
#[allow(dead_code)]
pub fn panel(theme: &UiTheme) -> Node {
    Node {
        padding: theme.panel_padding,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.gap_small),
        ..default()
    }
}

/// 绝对定位面板
#[allow(dead_code)]
pub fn panel_absolute(theme: &UiTheme, left: f32, top: f32, width: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        width: Val::Px(width),
        padding: theme.panel_padding,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.gap_small),
        ..default()
    }
}

/// 底部面板
pub fn panel_bottom(theme: &UiTheme, bottom: f32, left: f32, width: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(bottom),
        left: Val::Px(left),
        width: Val::Px(width),
        padding: UiRect::all(Val::Px(theme.gap_large)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.gap_medium),
        ..default()
    }
}

/// 右上角面板
pub fn panel_top_right(theme: &UiTheme, top: f32, right: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(top),
        right: Val::Px(right),
        width: Val::Px(width),
        height: Val::Px(height),
        padding: theme.panel_padding,
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

/// 文本标签
pub fn label(text: &str, font_size: f32, color: Color) -> (Text, TextFont, TextColor) {
    (
        Text::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
    )
}

/// 分隔线
#[allow(dead_code)]
pub fn divider(theme: &UiTheme) -> (Node, BackgroundColor) {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(theme.divider_color),
    )
}

/// 按钮节点（仅布局，不含交互逻辑）
pub fn button(theme: &UiTheme) -> Node {
    Node {
        padding: theme.button_padding,
        ..default()
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证 Widget 函数返回值
    // ✅ 符合领域规则：是 — 覆盖 UI Widget 公共接口
    // ✅ 确定性：是 — 硬编码默认值
    // ✅ 使用标准数据：是 — 使用标准 Default 实现
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ================================================

    use super::*;

    /// Test ID: UI-WDG-001
    /// Title: vbox 返回纵向布局节点
    ///
    /// Given: 默认 UiTheme
    /// When: 调用 vbox(&theme)
    /// Then: 返回 Column 方向节点，间距正确
    ///
    /// Assertions: flex_direction == Column, row_gap == gap_small
    #[test]
    fn vbox_返回纵向布局() {
        // Given
        let theme = UiTheme::default();

        // When
        let node = vbox(&theme);

        // Then
        assert_eq!(node.flex_direction, FlexDirection::Column);
        assert_eq!(node.row_gap, Val::Px(theme.gap_small));
    }

    /// Test ID: UI-WDG-002
    /// Title: hbox 返回横向布局节点
    ///
    /// Given: 默认 UiTheme
    /// When: 调用 hbox(&theme)
    /// Then: 返回 Row 方向节点，间距正确，居中对齐
    ///
    /// Assertions: flex_direction == Row, column_gap == gap_small, align_items == Center
    #[test]
    fn hbox_返回横向布局() {
        // Given
        let theme = UiTheme::default();

        // When
        let node = hbox(&theme);

        // Then
        assert_eq!(node.flex_direction, FlexDirection::Row);
        assert_eq!(node.column_gap, Val::Px(theme.gap_small));
        assert_eq!(node.align_items, AlignItems::Center);
    }

    /// Test ID: UI-WDG-003
    /// Title: panel 返回面板布局节点
    ///
    /// Given: 默认 UiTheme
    /// When: 调用 panel(&theme)
    /// Then: 返回 Column 方向节点，内边距正确
    ///
    /// Assertions: flex_direction == Column, padding == panel_padding
    #[test]
    fn panel_返回带内边距布局() {
        // Given
        let theme = UiTheme::default();

        // When
        let node = panel(&theme);

        // Then
        assert_eq!(node.flex_direction, FlexDirection::Column);
        assert_eq!(node.padding, theme.panel_padding);
    }

    /// Test ID: UI-WDG-004
    /// Title: label 返回正确的文本组件元组
    ///
    /// Given: 文本 "测试", 字号 16.0, 颜色 WHITE
    /// When: 调用 label("测试", 16.0, Color::WHITE)
    /// Then: 返回 (Text, TextFont, TextColor) 且字段正确
    ///
    /// Assertions: text 内容正确, font_size == 16.0, color == WHITE
    #[test]
    fn label_返回正确组件() {
        // Given
        let text = "测试";
        let font_size = 16.0;
        let color = Color::WHITE;

        // When
        let (text_component, font_component, color_component) = label(text, font_size, color);

        // Then
        assert_eq!(text_component.0, text);
        assert!((font_component.font_size - font_size).abs() < f32::EPSILON);
        assert_eq!(color_component.0, color);
    }

    /// Test ID: UI-WDG-005
    /// Title: divider 返回分隔线节点
    ///
    /// Given: 默认 UiTheme
    /// When: 调用 divider(&theme)
    /// Then: 返回 100% 宽度、1px 高度的节点，颜色正确
    ///
    /// Assertions: width == 100%, height == 1px, color == divider_color
    #[test]
    fn divider_返回水平分隔线() {
        // Given
        let theme = UiTheme::default();

        // When
        let (node, bg_color) = divider(&theme);

        // Then
        assert_eq!(node.width, Val::Percent(100.0));
        assert_eq!(node.height, Val::Px(1.0));
        assert_eq!(bg_color.0, theme.divider_color);
    }
}
