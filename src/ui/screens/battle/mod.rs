//! 战斗主界面（MVP）
//!
//! 全屏战斗 UI，将现有 Widget 组合成功能性的战斗界面布局：
//! 回合信息栏、战斗区域占位符、角色卡片、行动菜单和结束回合按钮。
//!
//! 仅使用原语层和 Widget 层工厂。不直接操作 Node/Button/Interaction。
//!
//! UI 树结构：
//!
//! ```text
//! Panel (Basic, full screen)
//!   ├── Text ("Turn: 3    Phase: Player Turn", Body)
//!   ├── Panel (Basic, 战斗区域占位符)
//!   ├── CharacterCard (Aria, Lv.5, HP/MP bars)
//!   ├── ActionMenu (Attack, Defend, Skill, Item, Wait)
//!   └── Button ("End Turn", Danger) -- BattleAction::EndTurn
//! ```

pub mod systems;

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_text};
use crate::ui::theme::Theme;
use crate::ui::widgets::action_menu::factory::spawn_action_menu;
use crate::ui::widgets::character_card::factory::spawn_character_card;

use systems::BattleAction;

/// 战斗界面标记组件
///
/// 用于场景管理清理（离开战斗界面时销毁所有携带此组件的实体）。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct BattleScreen;

/// 启动系统：生成战斗界面（MVP）
///
/// 创建全屏战斗 UI 树。所有元素通过原语/Widget 工厂创建
/// — 不直接操作 Node/Button/Interaction。
pub fn spawn_battle_screen(
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    // ── 1. Root panel ──
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(root).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::FlexStart,
            padding: UiRect::all(Val::Px(theme.spacing.md)),
            ..default()
        },
        BattleScreen,
    ));

    // ── 2. Turn info text bar ──
    let turn_info = spawn_text(
        &mut commands,
        &asset_server,
        &theme,
        "Turn: 3    Phase: Player Turn",
        TextVariant::Body,
    );
    commands.entity(turn_info).set_parent_in_place(root);

    // ── 3. Battle area placeholder panel ──
    let battle_area = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(battle_area).insert(Node {
        width: Val::Percent(100.0),
        height: Val::Px(300.0),
        margin: UiRect::vertical(Val::Px(theme.spacing.sm)),
        ..default()
    });
    commands.entity(battle_area).set_parent_in_place(root);

    // ── 4. Character card (Aria, Lv.5, 80/100 HP, 40/50 MP) ──
    let char_card = spawn_character_card(
        &mut commands,
        &asset_server,
        &theme,
        "Aria",
        5,
        80.0,
        100.0,
        40.0,
        50.0,
    );
    commands.entity(char_card).set_parent_in_place(root);

    // ── 5. Action menu (Attack, Defend, Skill, Item, Wait) ──
    let action_menu = spawn_action_menu(&mut commands, &theme);
    commands.entity(action_menu).set_parent_in_place(root);

    // ── 6. End Turn button (Danger variant) ──
    let end_turn_btn = spawn_localized_button(
        &mut commands,
        &theme,
        loc::ui::BATTLE_END_TURN,
        "End Turn",
        ButtonVariant::Danger,
    );
    commands.entity(end_turn_btn).insert(BattleAction::EndTurn);
    commands.entity(end_turn_btn).set_parent_in_place(root);
}

/// 清除系统：离开战斗时销毁所有战斗屏幕实体
pub fn despawn_battle_screen(mut commands: Commands, query: Query<Entity, With<BattleScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
