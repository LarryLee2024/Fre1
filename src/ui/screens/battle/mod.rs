//! 战斗主界面 — 9-zone 绝对定位布局
//!
//! 使用 Zone 容器替代单列 Column 布局。
//! 所有 Zone 是平级兄弟节点，通过 PositionType::Absolute 锚定到屏幕边缘。
//!
//! Zone 4 (Z4) 由 2D 相机处理，不在 UI 范围内。
//!
//! UI 树结构：
//!
//! ```text
//! Node (full screen, position_type: Relative)
//!   ├── Zone_Z1_TopLeft      (absolute, top-left):    TurnIndicator
//!   ├── Zone_Z2_TopCenter    (absolute, top):         PhaseText + TurnNumber
//!   ├── Zone_Z3_TopRight     (absolute, top-right):   UnitSummary [P2]
//!   ├── Zone_Z5_BottomLeft   (absolute, bottom-left): CharacterCard
//!   ├── Zone_Z6_BottomCenter (absolute, bottom):      ActionMenu
//!   ├── Zone_Z7_BottomRight  (absolute, bottom-right): SkillPanel [P1] + EndTurnButton
//!   └── Zone_Z8_BottomBar    (absolute, bottom bar):  TurnOrderBar [P2]
//! ```

pub mod layout;
pub mod systems;
pub mod visibility;

use bevy::prelude::*;

use crate::infra::localization::generated::loc;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
use crate::ui::primitives::panel::{components::PanelVariant, factory::spawn_panel};
use crate::ui::primitives::text::{components::TextVariant, factory::spawn_localized_text};
use crate::ui::theme::Theme;
use crate::ui::widgets::action_menu::factory::spawn_action_menu;
use crate::ui::widgets::character_card::factory::spawn_character_card;

use self::layout::{BattleZone, spawn_zone};
use systems::BattleAction;

/// 战斗界面标记组件
///
/// 用于场景管理清理（离开战斗界面时销毁所有携带此组件的实体）。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct BattleScreen;

/// 启动系统：生成战斗界面（9-zone 布局）
///
/// 创建全屏战斗 UI 树。所有元素通过原语/Widget 工厂创建
/// — 不直接操作 Node/Button/Interaction。
pub fn spawn_battle_screen(
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    // ── 0. Root full-screen container ──
    // TODO[P0][BATTLE][2026-07-21]: spawn_panel should accept sizing config so full-screen
    // width/height does not need to be set externally (see factory.rs). Currently the factory's
    // Node is used as-is; if absolute-positioned children need explicit parent bounds, add a
    // PanelVariant::FullScreen or sizing props to the factory instead of overriding Node here.
    let root = spawn_panel(&mut commands, &theme, PanelVariant::Basic);
    commands.entity(root).insert((
        BattleScreen,
        Name::new("BattleScreen"),
    ));

    // ── Z1: Top-Left — Turn Indicator ──
    let z1 = spawn_zone(&mut commands, &theme, BattleZone::Z1TopLeft);
    commands.entity(z1).set_parent_in_place(root);
    let turn_info = spawn_localized_text(
        &mut commands,
        &asset_server,
        &theme,
        "ui.battle.turn_bar",
        "-- / ----",
        TextVariant::Body,
    );
    commands.entity(turn_info).set_parent_in_place(z1);

    // ── Z2: Top-Center — Phase Text + Turn Number ──
    // TODO[P2][UI][2026-07-21]: Add PhaseText and TurnNumber widgets
    let z2 = spawn_zone(&mut commands, &theme, BattleZone::Z2TopCenter);
    commands.entity(z2).set_parent_in_place(root);

    // ── Z3: Top-Right — Unit Summary [P2] ──
    // TODO[P2][UI][2026-07-21]: Add UnitSummary widget
    let z3 = spawn_zone(&mut commands, &theme, BattleZone::Z3TopRight);
    commands.entity(z3).set_parent_in_place(root);

    // ── Z5: Bottom-Left — Character Card ──
    let z5 = spawn_zone(&mut commands, &theme, BattleZone::Z5BottomLeft);
    commands.entity(z5).set_parent_in_place(root);
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
    commands.entity(char_card).set_parent_in_place(z5);

    // ── Z6: Bottom-Center — Action Menu ──
    let z6 = spawn_zone(&mut commands, &theme, BattleZone::Z6BottomCenter);
    commands.entity(z6).set_parent_in_place(root);
    let action_menu = spawn_action_menu(&mut commands, &theme);
    commands.entity(action_menu).set_parent_in_place(z6);

    // ── Z7: Bottom-Right — End Turn Button + SkillPanel [P1] ──
    // TODO[P1][UI][2026-07-21]: Add SkillPanel widget
    let z7 = spawn_zone(&mut commands, &theme, BattleZone::Z7BottomRight);
    commands.entity(z7).set_parent_in_place(root);
    let end_turn_btn = spawn_localized_button(
        &mut commands,
        &theme,
        loc::ui::BATTLE_END_TURN,
        "End Turn",
        ButtonVariant::Danger,
    );
    commands.entity(end_turn_btn).insert(BattleAction::EndTurn);
    commands.entity(end_turn_btn).set_parent_in_place(z7);

    // ── Z8: Bottom Bar — Turn Order [P2] ──
    // TODO[P2][UI][2026-07-21]: Add TurnOrderBar widget
    let z8 = spawn_zone(&mut commands, &theme, BattleZone::Z8BottomBar);
    commands.entity(z8).set_parent_in_place(root);
}

/// 清除系统：离开战斗时销毁所有战斗屏幕实体
pub fn despawn_battle_screen(mut commands: Commands, query: Query<Entity, With<BattleScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
