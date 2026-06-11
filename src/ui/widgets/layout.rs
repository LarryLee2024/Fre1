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
