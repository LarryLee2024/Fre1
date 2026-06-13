/// 框架 UI 屏幕模块：全屏、独占的 UI 视图
///
/// 每个屏幕使用 Spawn/Despawn OnEnter/OnExit 模式：
/// - OnEnter(AppState::Xxx) 生成 UI Entity
/// - OnExit(AppState::Xxx)  清理 UI Entity
pub mod game_over;
pub mod level_select;
pub mod main_menu;

use bevy::prelude::*;

use crate::campaign::progress::{CampaignProgress, StageStatus};
use crate::campaign::registry::CampaignRegistry;
use crate::turn::AppState;
use crate::ui::events::UiCommand;

/// 框架屏幕插件
pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 ViewModel Resource
            .init_resource::<super::view_models::LevelSelectState>()
            .init_resource::<super::view_models::GameResultView>()
            // MainMenu 屏幕
            .add_systems(OnEnter(AppState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(OnExit(AppState::MainMenu), main_menu::cleanup_main_menu)
            .add_systems(
                Update,
                main_menu::handle_main_menu_buttons.run_if(in_state(AppState::MainMenu)),
            )
            // LevelSelect 屏幕
            .add_systems(
                OnEnter(AppState::LevelSelect),
                level_select::spawn_level_select,
            )
            .add_systems(
                OnExit(AppState::LevelSelect),
                level_select::cleanup_level_select,
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
                    game_over::update_game_result_view,
                    game_over::spawn_game_over_screen,
                ),
            )
            .add_systems(
                OnExit(AppState::GameOver),
                game_over::cleanup_game_over_screen,
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

/// 处理菜单相关的 UiCommand（非战斗状态）
///
/// 处理：StartGame, ConfirmStage, RetryStage, NextStage,
///       BackToLevelSelect, BackToMainMenu, QuitGame
/// SelectStage 和 BackToMainMenu 已在屏幕交互系统中处理。
fn handle_menu_commands(
    mut commands: Commands,
    mut events: MessageReader<UiCommand>,
    mut next_state: ResMut<NextState<AppState>>,
    mut campaign_progress: ResMut<CampaignProgress>,
    campaign_registry: Res<CampaignRegistry>,
    mut app_exit: MessageWriter<bevy::app::AppExit>,
) {
    for cmd in events.read() {
        match cmd {
            UiCommand::StartGame => {
                // 初始化战役进度
                *campaign_progress = CampaignProgress::initialize(&campaign_registry);
                next_state.set(AppState::LevelSelect);
            }
            UiCommand::SelectStage { stage_id } => {
                // 设置选中的关卡
                campaign_progress.current_stage = Some(stage_id.clone());
            }
            UiCommand::ConfirmStage => {
                // 验证关卡已解锁
                let can_enter = campaign_progress.current_stage.as_ref().is_some_and(|id| {
                    matches!(
                        campaign_progress.stage_status(id),
                        Some(StageStatus::Unlocked) | Some(StageStatus::Completed)
                    )
                });
                if can_enter {
                    // 重置 GameOverState 为 Playing（防止残留终态影响下次战斗）
                    commands.insert_resource(crate::turn::GameOverState::Playing);
                    next_state.set(AppState::InGame);
                }
            }
            UiCommand::RetryStage => {
                // 重玩当前关卡：先重置游戏状态，再进入 InGame
                commands.insert_resource(crate::turn::GameOverState::Playing);
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
