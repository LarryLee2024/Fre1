//! Module Name: BattleScreen — Battle main screen (MVP)
//!
//! Full-screen battle UI combining existing widgets into a functioning
//! combat screen layout: turn info bar, battle area placeholder,
//! character card, action menu, and an End Turn button.
//!
//! Uses primitives-layer factories and widget factories exclusively.
//! No direct Node/Button/Interaction manipulation outside factories.
//!
//! UI tree structure:
//!
//! ```text
//! Panel (Basic, full screen)
//!   ├── Text ("Turn: 3    Phase: Player Turn", Body)
//!   ├── Panel (Basic, battle area placeholder)
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

/// Battle screen marker component
///
/// Used for scene-management cleanup (despawn all entities carrying this
/// component when leaving the battle screen).
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct BattleScreen;

/// Startup System: spawns the battle screen (MVP)
///
/// Creates the full-screen battle UI tree. All elements are created
/// through primitives/widget factories -- no direct Node/Button/Interaction
/// manipulation.
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
    let end_turn_btn = spawn_localized_button(&mut commands, &theme, loc::ui::BATTLE_END_TURN, "End Turn", ButtonVariant::Danger);
    commands.entity(end_turn_btn).insert(BattleAction::EndTurn);
    commands.entity(end_turn_btn).set_parent_in_place(root);
}

/// 清除系统：离开战斗时销毁所有战斗屏幕实体
pub fn despawn_battle_screen(mut commands: Commands, query: Query<Entity, With<BattleScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
