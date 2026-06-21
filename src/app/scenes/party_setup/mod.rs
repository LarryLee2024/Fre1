//! Module Name: PartySetupScreen — 战前准备界面
//!
//! MVP: 只有一个 Quick Battle 按钮，未来会替换为完整的队伍编成界面。
//!
//! 使用 Primitives 层工厂函数 (spawn_panel, spawn_text, spawn_button) 创建 UI 元素。
//! Quick Battle 按钮点击后通过 StateTransitionQueue 切换到 Combat 场景。

use bevy::prelude::*;

use crate::shared::game_state::TransitionRequest;
use crate::ui::primitives::button::events::ButtonClicked;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_localized_text};
use crate::ui::theme::Theme;

use super::queue::StateTransitionQueue;

/// 战前准备屏幕标记组件
///
/// 用于场景管理系统的 despawn 清理。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct PartySetupScreen;

/// PartySetup 按钮动作标识
///
/// 作为 Component 挂载在按钮实体上，Observer 通过查询此组件识别被点击的按钮。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum PartySetupAction {
    /// 快速进入测试战斗（MVP 开发用）
    QuickBattle,
}

/// Startup System: 生成 PartySetup 界面
///
/// 创建一个居中布局面板，包含标题和 Quick Battle 按钮。
///
/// # UI 树结构
///
/// ```text
/// Panel (Basic, full screen)
///   └── Text ("Party Setup", Heading)
///   └── Button ("Quick Battle", Primary)
/// ```
pub fn spawn_party_setup(
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
        PartySetupScreen,
    ));

    // Title
    let title = spawn_localized_text(
        &mut commands,
        &asset_server,
        &theme,
        "ui.party.setup.title",
        "Party Setup",
        TextVariant::Heading,
    );
    commands.entity(title).set_parent_in_place(root);

    // Quick Battle button
    let btn = spawn_localized_button(
        &mut commands,
        &theme,
        "ui.party.setup.quick.battle",
        "Quick Battle",
        ButtonVariant::Primary,
    );
    commands.entity(btn).insert(PartySetupAction::QuickBattle);
    commands.entity(btn).set_parent_in_place(root);
}

/// OnExit System: 离开 PartySetup 时销毁所有 PartySetup 实体
pub fn despawn_party_setup(mut commands: Commands, query: Query<Entity, With<PartySetupScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Observer: 处理 PartySetup 按钮点击
///
/// 当 primitives 层的 button_interaction_system 触发 ButtonClicked 事件时，
/// 检查按钮实体是否挂载了 PartySetupAction 组件，匹配后执行对应操作。
pub fn on_party_setup_button(
    on: On<ButtonClicked>,
    query: Query<&PartySetupAction>,
    mut queue: ResMut<StateTransitionQueue>,
) {
    let entity = on.event().entity;
    let Ok(action) = query.get(entity) else {
        return;
    };
    match action {
        PartySetupAction::QuickBattle => {
            info!(target: "app", "QuickBattle button — transitioning to Combat");
            queue.push(TransitionRequest::Change(
                crate::shared::game_state::GameState::Combat,
            ));
        }
    }
}
