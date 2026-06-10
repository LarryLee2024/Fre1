// 操作提示面板：显示快捷键提示

use crate::turn::AppState;
use crate::ui::theme::UiTheme;
use bevy::prelude::*;

/// 生成操作提示
pub fn spawn_action_hint(mut commands: Commands, theme: Res<UiTheme>) {
    commands.spawn((
        Text::new("左键选择/移动 | 右键取消 | ESC 关闭面板/取消 | E 结束回合"),
        TextFont {
            font_size: theme.font_small,
            ..default()
        },
        TextColor(theme.text_secondary),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(theme.gap_large + theme.font_large + theme.gap_small),
            left: Val::Px(theme.gap_large),
            ..default()
        },
    ));
}

/// 操作提示插件
pub struct ActionHintPlugin;

impl Plugin for ActionHintPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_action_hint.in_set(GameSet::Ui),
        );
    }
}
