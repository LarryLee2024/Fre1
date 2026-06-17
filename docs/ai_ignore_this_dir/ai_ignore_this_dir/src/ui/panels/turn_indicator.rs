// 回合提示面板：显示当前回合数、胜负判定

use crate::core::turn::{AppState, GameOverState};
use crate::infrastructure::localization::{CurrentLocale, LocalizationService};
use crate::ui::theme::UiTheme;
use crate::ui::view_models::TurnInfoView;
use bevy::prelude::*;

/// 回合提示文本
#[derive(Component)]
pub struct TurnIndicator;

/// 生成回合提示
pub fn spawn_turn_indicator(
    mut commands: Commands,
    theme: Res<UiTheme>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    let round_text = localization.resolve("ui.turn_indicator.round", &locale.0, None);
    commands
        .spawn((
            Text::new(round_text),
            TextFont {
                font_size: theme.font_large,
                ..default()
            },
            TextColor(theme.text_primary),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(theme.gap_large),
                left: Val::Px(theme.gap_large),
                ..default()
            },
            TurnIndicator,
        ))
        .insert(Name::new("TurnIndicator"));
}

/// 更新回合提示（读取 TurnInfoView，AGI驱动不再分阵营阶段）
pub fn update_turn_indicator(
    turn_view: Res<TurnInfoView>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    if turn_view.is_changed() {
        for mut text in &mut query {
            **text = localization.resolve("ui.turn_indicator.round", &locale.0, None);
        }
    }
}

/// 检查胜负条件（读取 GameOverState ViewModel）
pub fn check_game_over(
    game_over: Res<GameOverState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut turn_indicator: Query<&mut Text, With<TurnIndicator>>,
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    if game_over.is_changed() {
        match *game_over {
            GameOverState::Victory => {
                for mut text in &mut turn_indicator {
                    **text = localization.resolve("ui.turn_indicator.victory", &locale.0, None);
                }
                next_app_state.set(AppState::GameOver);
            }
            GameOverState::Defeat => {
                for mut text in &mut turn_indicator {
                    **text = localization.resolve("ui.turn_indicator.defeat", &locale.0, None);
                }
                next_app_state.set(AppState::GameOver);
            }
            GameOverState::Playing => {}
        }
    }
}

/// 回合提示插件
pub struct TurnIndicatorPlugin;

impl Plugin for TurnIndicatorPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::GameSet;
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_turn_indicator.in_set(GameSet::Ui),
        )
        .add_systems(
            Update,
            (update_turn_indicator, check_game_over).run_if(in_state(AppState::InGame)),
        );
    }
}
