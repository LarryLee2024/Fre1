// 战斗日志面板：可折叠、可调整大小、可滚动

use crate::battle::{
    CombatLogCollapsed, CombatLogContent, CombatLogPanel, CombatLogResizeHandle, CombatLogToggle,
};
use crate::turn::AppState;
use crate::ui::theme::UiTheme;
use crate::ui::widgets::layout::*;
use bevy::prelude::*;

/// 生成战斗日志面板
pub fn spawn_combat_log_panel(mut commands: Commands, theme: Res<UiTheme>) {
    commands
        .spawn((
            panel_top_right(
                &theme,
                theme.gap_large,
                theme.gap_large,
                theme.log_panel_width,
                theme.log_panel_height,
            ),
            BackgroundColor(theme.panel_bg),
            CombatLogPanel,
        ))
        .with_children(|parent| {
            // 标题行
            parent.spawn(hbox(&theme)).with_children(|row| {
                row.spawn((
                    Button,
                    Text::new("▼"),
                    TextFont {
                        font_size: theme.font_small,
                        ..default()
                    },
                    TextColor(theme.text_primary),
                    Node {
                        padding: UiRect::px(theme.gap_small, theme.gap_small, 2.0, 2.0),
                        ..default()
                    },
                    CombatLogToggle,
                ));
                row.spawn((
                    Text::new("战斗日志"),
                    TextFont {
                        font_size: theme.font_small,
                        ..default()
                    },
                    TextColor(theme.text_secondary),
                ));
            });
            // 内容容器
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    flex_grow: 1.0,
                    ..default()
                },
                ScrollPosition::default(),
                CombatLogContent,
            ));
            // 拖拽手柄
            parent.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    margin: UiRect {
                        left: Val::Auto,
                        bottom: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(2.0),
                    },
                    ..default()
                },
                Button,
                Text::new("⇲"),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                CombatLogResizeHandle,
            ));
        });
}

/// 战斗日志折叠/展开切换
pub fn toggle_combat_log(
    mut collapsed: ResMut<CombatLogCollapsed>,
    toggle_query: Query<&Interaction, (With<CombatLogToggle>, Changed<Interaction>)>,
    mut toggle_text: Query<&mut Text, With<CombatLogToggle>>,
    mut content_vis: Query<&mut Visibility, With<CombatLogContent>>,
) {
    for interaction in &toggle_query {
        if *interaction == Interaction::Pressed {
            collapsed.0 = !collapsed.0;
            let arrow = if collapsed.0 { "▶" } else { "▼" };
            for mut text in &mut toggle_text {
                **text = arrow.to_string();
            }
            for mut vis in &mut content_vis {
                *vis = if collapsed.0 {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}

/// 战斗日志面板插件
pub struct CombatLogPanelPlugin;

impl Plugin for CombatLogPanelPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_combat_log_panel.in_set(GameSet::Ui),
        )
        .add_systems(Update, toggle_combat_log.run_if(in_state(AppState::InGame)));
    }
}
