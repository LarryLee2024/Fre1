// 资源条 Widget：HP/MP/STA 进度条的可复用构建块
// 从 hud.rs 提取，消除硬编码

use crate::ui::theme::UiTheme;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

/// 生成资源条行：标签 + 进度条背景 + 填充 + 数值文本
pub fn spawn_resource_bar(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    theme: &UiTheme,
    label: &str,
    fill_color: Color,
    fill_marker: impl Component,
    text_label: impl Component,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.gap_medium),
                ..default()
            },
        ))
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(theme.bar_label_width),
                    ..default()
                },
            ));
            // 进度条背景
            row.spawn((
                Node {
                    width: Val::Px(theme.bar_width),
                    height: Val::Px(theme.bar_height),
                    ..default()
                },
                BackgroundColor(theme.bar_bg),
            ))
            .with_children(|bar| {
                // 进度条填充
                bar.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(theme.bar_height),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                    fill_marker,
                ));
            });
            // 数值文本
            row.spawn((
                Text::new("0/0"),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(Color::WHITE),
                text_label,
            ));
        });
}
