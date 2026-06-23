//! Module Name: GameOverScreen — 游戏结束画面
//!
//! 显示 "Game Over" 标题和 "Main Menu" 按钮。
//! 按钮点击后通过 StateTransitionQueue 切换到 MainMenu 状态。

use bevy::prelude::*;

use crate::shared::game_state::TransitionRequest;
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_localized_text};
use crate::ui::theme::Theme;

use super::queue::StateTransitionQueue;
use crate::infra::localization::generated::loc;
use crate::shared::game_state::GameState;

/// 游戏结束屏幕标记组件
///
/// 用于场景管理系统的 despawn 清理。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct GameOverScreen;

/// GameOverScreen 按钮动作标识
///
/// 作为 Component 挂载在按钮实体上，Observer 通过查询此组件识别被点击的按钮。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum GameOverScreenAction {
    /// 返回主菜单
    MainMenu,
}

/// Startup System: 生成游戏结束界面
///
/// 创建一个居中布局面板，包含 Game Over 标题和 Main Menu 按钮。
///
/// # UI 树结构
///
/// ```text
/// Panel (Basic, full screen)
///   └── Text ("tutorial.defeat", Title)
///   └── Button ("ui.main.menu", Primary)
/// ```
pub fn spawn_game_over_screen(
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    // ── 0. Root panel ──
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(theme.spacing.lg),
            ..default()
        },
        GameOverScreen,
    ));

    // Game Over title
    let title = spawn_localized_text(
        &mut commands,
        &asset_server,
        &theme,
        loc::tutorial::DEFEAT,
        "Game Over",
        TextVariant::Title,
    );
    commands.entity(title).set_parent_in_place(root);

    // Main Menu button
    let btn = spawn_localized_button(
        &mut commands,
        &theme,
        loc::ui::MAIN_MENU,
        "Main Menu",
        ButtonVariant::Primary,
    );
    commands.entity(btn).insert(GameOverScreenAction::MainMenu);
    commands.entity(btn).set_parent_in_place(root);
}

/// OnExit System: 离开 GameOverScreen 时销毁所有 GameOverScreen 实体
pub fn despawn_game_over_screen(
    mut commands: Commands,
    query: Query<Entity, With<GameOverScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Observer: 处理 GameOverScreen 按钮点击
///
/// 当 primitives 层的 button_interaction_system 触发 ButtonClicked 事件时，
/// 检查按钮实体是否挂载了 GameOverScreenAction 组件，匹配后执行对应操作。
pub fn on_game_over_screen_button(
    on: On<ButtonClicked>,
    query: Query<&GameOverScreenAction>,
    mut queue: ResMut<StateTransitionQueue>,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        return;
    };
    match action {
        GameOverScreenAction::MainMenu => {
            info!(target: "app", "GameOverScreen: transitioning to MainMenu");
            queue.push(TransitionRequest::Change(GameState::MainMenu));
        }
    }
}
