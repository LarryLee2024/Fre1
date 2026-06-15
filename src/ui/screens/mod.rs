/// 框架 UI 屏幕模块：全屏、独占的 UI 视图
///
/// 每个屏幕使用 Spawn/Despawn OnEnter/OnExit 模式：
/// - OnEnter(AppState::Xxx) 生成 UI Entity
/// - OnExit(AppState::Xxx)  清理 UI Entity
pub mod game_over;
pub mod level_select;
pub mod main_menu;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::PrimaryEguiContext;

use crate::core::campaign::state::{CampaignProgress, StageStatus};
use crate::core::campaign::registry::CampaignRegistry;
use crate::core::map::LevelRegistry;
use crate::core::turn::AppState;
use crate::ui::events::UiCommand;

/// 框架屏幕插件
pub struct ScreensPlugin;

/// 菜单/结算屏幕共用的 UI 相机标记
#[derive(Component)]
struct MenuCamera;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 ViewModel Resource
            .init_resource::<super::view_models::LevelSelectState>()
            .init_resource::<super::view_models::GameResultView>()
            // MainMenu 屏幕
            .add_systems(
                OnEnter(AppState::MainMenu),
                (spawn_menu_camera, main_menu::spawn_main_menu),
            )
            .add_systems(
                OnExit(AppState::MainMenu),
                (despawn_menu_camera, main_menu::cleanup_main_menu),
            )
            .add_systems(
                Update,
                main_menu::handle_main_menu_buttons.run_if(in_state(AppState::MainMenu)),
            )
            // LevelSelect 屏幕
            .add_systems(
                OnEnter(AppState::LevelSelect),
                (spawn_menu_camera, level_select::spawn_level_select),
            )
            .add_systems(
                OnExit(AppState::LevelSelect),
                (despawn_menu_camera, level_select::cleanup_level_select),
            )
            .add_systems(
                Update,
                (
                    level_select::update_level_select_view,
                    level_select::handle_level_select_interaction,
                )
                    .run_if(in_state(AppState::LevelSelect)),
            )
            // GameOver 屏幕
            .add_systems(
                OnEnter(AppState::GameOver),
                (
                    spawn_menu_camera,
                    game_over::update_game_result_view,
                    game_over::spawn_game_over_screen,
                ),
            )
            .add_systems(
                OnExit(AppState::GameOver),
                (despawn_menu_camera, game_over::cleanup_game_over_screen),
            )
            .add_systems(
                Update,
                game_over::handle_game_over_interaction.run_if(in_state(AppState::GameOver)),
            )
            // 菜单命令处理（非战斗状态）
            .add_systems(
                Update,
                handle_menu_commands.run_if(not(in_state(AppState::InGame))),
            );
    }
}

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera, IsDefaultUiCamera, PrimaryEguiContext));
}

fn despawn_menu_camera(mut commands: Commands, cameras: Query<Entity, With<MenuCamera>>) {
    for entity in &cameras {
        commands.entity(entity).despawn();
    }
}

/// 处理菜单相关的 UiCommand（非战斗状态）
///
/// 处理：StartGame, ContinueGame, ConfirmStage, RetryStage, NextStage,
///       BackToLevelSelect, BackToMainMenu, QuitGame
/// SelectStage 和 BackToMainMenu 已在屏幕交互系统中处理。
fn handle_menu_commands(
    mut commands: Commands,
    mut events: MessageReader<UiCommand>,
    mut next_state: ResMut<NextState<AppState>>,
    mut campaign_progress: ResMut<CampaignProgress>,
    campaign_registry: Res<CampaignRegistry>,
    level_registry: Res<LevelRegistry>,
    mut app_exit: MessageWriter<bevy::app::AppExit>,
) {
    for cmd in events.read() {
        match cmd {
            UiCommand::StartGame => {
                // 初始化战役进度
                *campaign_progress = CampaignProgress::initialize(&campaign_registry);
                next_state.set(AppState::LevelSelect);
            }
            UiCommand::ContinueGame => {
                // 继续已有战役：直接进入关卡选择
                // CampaignProgress 已由 campaign 插件初始化，无需重复初始化
                next_state.set(AppState::LevelSelect);
            }
            UiCommand::SelectStage { stage_id } => {
                // 设置选中的关卡
                campaign_progress.current_stage = Some(stage_id.clone());
            }
            UiCommand::ConfirmStage => {
                // 验证关卡已解锁且 level_id 有效（FORBIDDEN-7）
                let can_enter = campaign_progress.current_stage.as_ref().is_some_and(|id| {
                    // 1. 检查 stage 状态为 Unlocked 或 Completed
                    let status_ok = matches!(
                        campaign_progress.stage_status(id),
                        Some(StageStatus::Unlocked) | Some(StageStatus::Completed)
                    );
                    if !status_ok {
                        return false;
                    }
                    // 2. 验证 stage 存在于 CampaignRegistry 中
                    let stage_exists = campaign_registry
                        .get(&campaign_progress.campaign_id)
                        .and_then(|c| c.stages.iter().find(|s| s.id == *id))
                        .is_some();
                    if !stage_exists {
                        bevy::log::error!(
                            target: "campaign",
                            stage_id = %id,
                            "ConfirmStage: stage 在 CampaignRegistry 中不存在"
                        );
                        return false;
                    }
                    // 3. 验证 stage 引用的 level_id 存在于 LevelRegistry 中
                    let level_valid = campaign_registry
                        .get(&campaign_progress.campaign_id)
                        .and_then(|c| c.stages.iter().find(|s| s.id == *id))
                        .map(|stage| level_registry.get(&stage.level_id).is_some())
                        .unwrap_or(false);
                    if !level_valid {
                        bevy::log::error!(
                            target: "campaign",
                            stage_id = %id,
                            "ConfirmStage: level_id 在 LevelRegistry 中不存在"
                        );
                        return false;
                    }
                    true
                });
                if can_enter {
                    // 重置 GameOverState 为 Playing（防止残留终态影响下次战斗）
                    commands.insert_resource(crate::core::turn::GameOverState::Playing);
                    next_state.set(AppState::InGame);
                }
            }
            UiCommand::RetryStage => {
                // 重玩当前关卡：先重置游戏状态，再进入 InGame
                commands.insert_resource(crate::core::turn::GameOverState::Playing);
                next_state.set(AppState::InGame);
            }
            UiCommand::NextStage => {
                // 下一关：返回关卡选择（进度已在 progression.rs 中更新）
                next_state.set(AppState::LevelSelect);
            }
            UiCommand::BackToLevelSelect => {
                next_state.set(AppState::LevelSelect);
            }
            UiCommand::BackToMainMenu => {
                next_state.set(AppState::MainMenu);
            }
            UiCommand::QuitGame => {
                app_exit.write(bevy::app::AppExit::Success);
            }
            _ => {}
        }
    }
}
