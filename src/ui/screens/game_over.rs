/// 游戏结果屏幕
use bevy::prelude::*;

use crate::assets::CnFont;
use crate::campaign::progress::CampaignProgress;
use crate::campaign::registry::CampaignRegistry;
use crate::map::LevelRegistry;
use crate::turn::GameOverState;
use crate::turn::TurnState;
use crate::ui::events::UiCommand;
use crate::ui::theme::UiTheme;
use crate::ui::view_models::{GameOutcome, GameResultView};

/// 游戏结果屏幕标记组件
#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct RetryButton;

#[derive(Component)]
pub struct NextStageButton;

#[derive(Component)]
pub struct BackToSelectButton;

/// 更新游戏结果 ViewModel（在进入 GameOver 状态时调用）
pub fn update_game_result_view(
    game_over: Res<GameOverState>,
    campaign_progress: Res<CampaignProgress>,
    campaign_registry: Res<CampaignRegistry>,
    level_registry: Res<LevelRegistry>,
    turn_state: Res<TurnState>,
    mut view: ResMut<GameResultView>,
) {
    let result = match *game_over {
        GameOverState::Victory => GameOutcome::Victory,
        GameOverState::Defeat => GameOutcome::Defeat,
        GameOverState::Playing => GameOutcome::Victory, // fallback, shouldn't happen
    };

    // 通过 CampaignRegistry 查找当前 stage 对应的关卡名称
    let stage_name = campaign_progress
        .current_stage
        .as_ref()
        .and_then(|current_id| {
            let campaign = campaign_registry.get(&campaign_progress.campaign_id)?;
            let stage = campaign.stages.iter().find(|s| s.id == *current_id)?;
            let level = level_registry.get(&stage.level_id)?;
            Some(level.name.clone())
        })
        .unwrap_or_default();

    // 从 TurnState 获取当前回合数
    let turn_count = turn_state.turn_number;

    let has_next_stage = match result {
        GameOutcome::Victory => match campaign_progress.current_stage.as_ref() {
            Some(current) => match campaign_registry.get(&campaign_progress.campaign_id) {
                Some(campaign) => campaign
                    .stages
                    .iter()
                    .position(|s| s.id == *current)
                    .map_or(false, |pos| pos + 1 < campaign.stages.len()),
                None => false,
            },
            None => false,
        },
        GameOutcome::Defeat => false,
    };

    view.result = result;
    view.turn_count = turn_count;
    view.stage_name = stage_name;
    view.has_next_stage = has_next_stage;
}

/// 生成游戏结果屏幕
pub fn spawn_game_over_screen(
    mut commands: Commands,
    theme: Res<UiTheme>,
    cn_font: Res<CnFont>,
    view: Res<GameResultView>,
) {
    let title_font = cn_font.text_font(theme.font_title);
    let subtitle_font = cn_font.text_font(theme.font_subtitle);
    let button_font = cn_font.text_font(theme.font_menu);

    let (result_text, result_color) = match view.result {
        GameOutcome::Victory => ("胜利！", theme.victory_color),
        GameOutcome::Defeat => ("失败...", theme.defeat_color),
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme.menu_bg),
            GameOverScreen,
        ))
        .with_children(|parent| {
            // 结果标题
            parent.spawn((
                Text::new(result_text),
                title_font.clone(),
                TextColor(result_color),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            // 关卡名
            if !view.stage_name.is_empty() {
                parent.spawn((
                    Text::new(&view.stage_name),
                    subtitle_font.clone(),
                    TextColor(theme.text_primary),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));
            }

            // 回合数
            if view.turn_count > 0 {
                parent.spawn((
                    Text::new(format!("回合数：{}", view.turn_count)),
                    subtitle_font.clone(),
                    TextColor(theme.text_secondary),
                    Node {
                        margin: UiRect::bottom(Val::Px(48.0)),
                        ..default()
                    },
                ));
            }

            // 按钮区域
            // 胜利时显示"下一关"和"重玩"
            if view.result == GameOutcome::Victory && view.has_next_stage {
                spawn_game_over_button(parent, "下一关", &theme, &button_font, NextStageButton);
            }

            spawn_game_over_button(parent, "重玩", &theme, &button_font, RetryButton);
            spawn_game_over_button(parent, "返回选关", &theme, &button_font, BackToSelectButton);
        });
}

/// 生成游戏结束按钮
fn spawn_game_over_button(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    label: &str,
    theme: &UiTheme,
    font: &TextFont,
    marker: impl Component,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(theme.menu_button_width),
                height: Val::Px(theme.menu_button_height),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(theme.menu_button_bg),
            marker,
        ))
        .with_child((
            Text::new(label),
            font.clone(),
            TextColor(theme.text_primary),
        ));
}

/// 处理游戏结果屏幕交互
pub fn handle_game_over_interaction(
    mut cmd_events: MessageWriter<UiCommand>,
    buttons: Query<(&Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    retry_query: Query<Entity, With<RetryButton>>,
    next_query: Query<Entity, With<NextStageButton>>,
    back_query: Query<Entity, With<BackToSelectButton>>,
) {
    for (interaction, entity) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if retry_query.contains(entity) {
            cmd_events.write(UiCommand::RetryStage);
        } else if next_query.contains(entity) {
            cmd_events.write(UiCommand::NextStage);
        } else if back_query.contains(entity) {
            cmd_events.write(UiCommand::BackToLevelSelect);
        }
    }
}

/// 清理游戏结果屏幕
pub fn cleanup_game_over_screen(
    mut commands: Commands,
    screens: Query<Entity, With<GameOverScreen>>,
) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}
