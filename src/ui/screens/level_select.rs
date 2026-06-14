/// 关卡选择屏幕
use bevy::prelude::*;

use crate::assets::CnFont;
use crate::campaign::progress::{CampaignProgress, StageStatus};
use crate::campaign::registry::CampaignRegistry;
use crate::map::LevelRegistry;
use crate::ui::events::UiCommand;
use crate::ui::theme::UiTheme;
use crate::ui::view_models::{LevelSelectState, StageEntry};

/// 关卡选择屏幕标记组件
#[derive(Component)]
pub struct LevelSelectScreen;

#[derive(Component)]
struct StageCard;

#[derive(Component)]
pub struct StageCardId(String);

#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
pub struct ConfirmButton;

/// 生成关卡选择屏幕
pub fn spawn_level_select(
    mut commands: Commands,
    theme: Res<UiTheme>,
    cn_font: Res<CnFont>,
    campaign_registry: Res<CampaignRegistry>,
    level_registry: Res<LevelRegistry>,
    campaign_progress: Res<CampaignProgress>,
    mut view: ResMut<LevelSelectState>,
) {
    // 构建 ViewModel
    let campaign = campaign_registry.first();
    let campaign_name = campaign.map(|c| c.name.clone()).unwrap_or_default();

    let mut stages: Vec<StageEntry> = Vec::new();
    if let Some(ref campaign) = campaign {
        for stage in &campaign.stages {
            let status = campaign_progress
                .stage_status(&stage.id)
                .cloned()
                .unwrap_or(StageStatus::Locked);
            let level_name = level_registry
                .get(&stage.level_id)
                .map(|l| l.name.clone())
                .unwrap_or_default();
            let level_description = String::new();
            stages.push(StageEntry {
                stage_id: stage.id.clone(),
                level_name,
                status,
                level_description,
            });
        }
    }

    view.campaign_name = campaign_name.clone();
    view.stages = stages.clone();
    view.selected_stage = campaign_progress.current_stage.clone();

    // 渲染 UI
    let title_font = cn_font.text_font(theme.font_subtitle);
    let button_font = cn_font.text_font(theme.font_menu);
    let small_font = cn_font.text_font(theme.font_small);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme.menu_bg),
            LevelSelectScreen,
        ))
        .with_children(|parent| {
            // 顶部：战役名 + 返回按钮
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },))
                .with_children(|top| {
                    // 战役名
                    top.spawn((
                        Text::new(campaign_name.clone()),
                        title_font.clone(),
                        TextColor(theme.menu_title_color),
                    ));
                    // 返回按钮
                    top.spawn((
                        Button,
                        Node {
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(theme.menu_button_bg),
                        BackButton,
                    ))
                    .with_child((
                        Text::new("← 返回"),
                        button_font.clone(),
                        TextColor(theme.text_primary),
                    ));
                });

            // 中间：关卡卡片网格
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    row_gap: Val::Px(16.0),
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(32.0)),
                    ..default()
                },))
                .with_children(|grid| {
                    for stage in &stages {
                        spawn_stage_card(grid, stage, &theme, &small_font);
                    }
                });

            // 底部：进入战斗按钮
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(16.0)),
                    ..default()
                },))
                .with_children(|bottom| {
                    let can_enter = view
                        .selected_stage
                        .as_ref()
                        .and_then(|id| view.stages.iter().find(|s| &s.stage_id == id))
                        .map(|s| {
                            s.status == StageStatus::Unlocked || s.status == StageStatus::Completed
                        })
                        .unwrap_or(false);

                    bottom
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(theme.menu_button_width),
                                height: Val::Px(theme.menu_button_height),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(if can_enter {
                                theme.menu_button_bg
                            } else {
                                Color::srgba(0.1, 0.1, 0.1, 0.5)
                            }),
                            ConfirmButton,
                        ))
                        .with_child((
                            Text::new("进入战斗"),
                            button_font.clone(),
                            TextColor(if can_enter {
                                theme.text_primary
                            } else {
                                theme.text_secondary
                            }),
                        ));
                });
        });
}

/// 生成关卡卡片
fn spawn_stage_card(
    parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    stage: &StageEntry,
    theme: &UiTheme,
    font: &TextFont,
) {
    let status_color = match stage.status {
        StageStatus::Locked => theme.stage_locked_color,
        StageStatus::Unlocked => theme.stage_unlocked_color,
        StageStatus::Completed => theme.stage_completed_color,
    };

    let status_label = stage.status.label();

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(theme.stage_card_width),
                height: Val::Px(theme.stage_card_height),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(theme.menu_button_bg),
            BorderColor::all(status_color),
            StageCard,
            StageCardId(stage.stage_id.clone()),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(&stage.level_name),
                font.clone(),
                TextColor(theme.text_primary),
            ));
            card.spawn((
                Text::new(status_label),
                font.clone(),
                TextColor(status_color),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
}

/// 更新关卡选择 ViewModel（在 CampaignProgress 变化时刷新）
pub fn update_level_select_view(
    campaign_progress: Res<CampaignProgress>,
    mut view: ResMut<LevelSelectState>,
) {
    if campaign_progress.is_changed() {
        view.selected_stage = campaign_progress.current_stage.clone();
    }
}

/// 处理关卡选择交互
pub fn handle_level_select_interaction(
    mut cmd_events: MessageWriter<UiCommand>,
    buttons: Query<(&Interaction, Entity), (Changed<Interaction>, With<Button>)>,
    card_query: Query<&StageCardId>,
    back_query: Query<Entity, With<BackButton>>,
    confirm_query: Query<Entity, With<ConfirmButton>>,
) {
    for (interaction, entity) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if back_query.contains(entity) {
            cmd_events.write(UiCommand::BackToMainMenu);
        } else if confirm_query.contains(entity) {
            cmd_events.write(UiCommand::ConfirmStage);
        } else if let Ok(card_id) = card_query.get(entity) {
            cmd_events.write(UiCommand::SelectStage {
                stage_id: card_id.0.clone(),
            });
        }
    }
}

/// 清理关卡选择屏幕
pub fn cleanup_level_select(
    mut commands: Commands,
    screens: Query<Entity, With<LevelSelectScreen>>,
) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}
