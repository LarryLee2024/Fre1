// 操作提示面板：显示快捷键提示

use crate::core::turn::AppState;
use crate::infrastructure::localization::{CurrentLocale, LocalizationService};
use crate::ui::theme::UiTheme;
use bevy::prelude::*;

#[derive(Component)]
pub struct ActionHint;

/// 生成操作提示
pub fn spawn_action_hint(
    mut commands: Commands,
    theme: Res<UiTheme>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let hint_text = localization.resolve("ui.action_hint.default", &locale.0, None);
    commands
        .spawn((
            Text::new(hint_text),
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
        ))
        .insert(Name::new("ActionHint"))
        .insert(ActionHint);
}

/// 操作提示插件
pub struct ActionHintPlugin;

impl Plugin for ActionHintPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_action_hint.in_set(GameSet::Ui),
        );
    }
}
