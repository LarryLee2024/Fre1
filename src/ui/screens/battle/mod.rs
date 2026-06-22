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
use crate::ui::binding::{Dirty, UiBinding};
use crate::ui::localization::text_keys::BATTLE_PHASE_PLAYER;
use crate::ui::primitives::button::{components::ButtonVariant, factory::spawn_localized_button};
// 注意: 根节点直接 spawn 而非通过 spawn_panel，因为根不需要背景色。
// PanelVariant::Basic 使用不透明的 surface_primary 背景色，
// 会遮挡底层的 2D 精灵（棋盘网格 + 棋子）。
// 各 Zone 容器有各自的 Panel/背景，根只需做布局容器。
use crate::ui::primitives::text::{
    components::TextVariant,
    factory::{spawn_localized_text, spawn_text},
};
use crate::ui::theme::Theme;
use crate::ui::widgets::action_menu::factory::spawn_action_menu;
use crate::ui::widgets::character_card::factory::spawn_character_card;
use crate::ui::widgets::skill_panel::factory::spawn_skill_panel;
use crate::ui::widgets::turn_order_bar::factory::spawn_turn_order_bar;
use crate::ui::widgets::unit_summary::factory::spawn_unit_summary;

use crate::ui::view_models::battle_hud::{BattleHudData, BattleHudVm};

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
    data: Res<BattleHudData>,
) {
    // ── 0. 根节点全屏容器（无背景色，让 2D 精灵可见）──
    // 不使用 spawn_panel(PanelVariant::Basic)，因为其 opaque 背景色会遮挡
    // 底层的棋盘网格和棋子。
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BattleScreen,
            Pickable::IGNORE,
            Name::new("BattleScreen"),
            Dirty::<BattleHudVm>::default(),
        ))
        .id();

    // ── Z1: 左上区 — 回合指示器 ──
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

    // ── Z2: 中上区 — 阶段文本 + 回合数 ──
    let z2 = spawn_zone(&mut commands, &theme, BattleZone::Z2TopCenter);
    commands.entity(z2).set_parent_in_place(root);

    let phase_text = spawn_localized_text(
        &mut commands,
        &asset_server,
        &theme,
        BATTLE_PHASE_PLAYER,
        "Player Phase",
        TextVariant::Body,
    );
    commands.entity(phase_text).set_parent_in_place(z2);

    let turn_text = spawn_text(
        &mut commands,
        &asset_server,
        &theme,
        "Turn 1",
        TextVariant::Body,
    );
    commands.entity(turn_text).set_parent_in_place(z2);

    // ── Z3: 右上区 — 单位摘要 ──
    let z3 = spawn_zone(&mut commands, &theme, BattleZone::Z3TopRight);
    commands.entity(z3).set_parent_in_place(root);
    let unit_summary = spawn_unit_summary(
        &mut commands,
        &asset_server,
        &theme,
        &data.character_name,
        data.level,
        data.hp_current,
        data.hp_max,
    );
    commands.entity(unit_summary).set_parent_in_place(z3);

    // ── Z5: 左下区 — 角色卡 ──
    let z5 = spawn_zone(&mut commands, &theme, BattleZone::Z5BottomLeft);
    commands.entity(z5).set_parent_in_place(root);
    let char_card = spawn_character_card(
        &mut commands,
        &asset_server,
        &theme,
        &data.character_name,
        data.level,
        data.hp_current,
        data.hp_max,
        data.mp_current,
        data.mp_max,
    );
    commands.entity(char_card).insert(UiBinding::Hp);
    commands.entity(char_card).set_parent_in_place(z5);

    // ── Z6: 中下区 — 行动菜单 ──
    let z6 = spawn_zone(&mut commands, &theme, BattleZone::Z6BottomCenter);
    commands.entity(z6).set_parent_in_place(root);
    let action_menu = spawn_action_menu(&mut commands, &theme);
    commands.entity(action_menu).set_parent_in_place(z6);

    // ── Z7: 右下区 — 技能面板 + 结束回合按钮 ──
    let z7 = spawn_zone(&mut commands, &theme, BattleZone::Z7BottomRight);
    commands.entity(z7).set_parent_in_place(root);

    // 技能面板（包含攻击、火球术、治疗三个技能槽位）
    let skill_panel = spawn_skill_panel(&mut commands, &asset_server, &theme);
    commands.entity(skill_panel).set_parent_in_place(z7);

    // 结束回合按钮
    let end_turn_btn = spawn_localized_button(
        &mut commands,
        &theme,
        loc::ui::BATTLE_END_TURN,
        "End Turn",
        ButtonVariant::Danger,
    );
    commands.entity(end_turn_btn).insert(BattleAction::EndTurn);
    commands.entity(end_turn_btn).set_parent_in_place(z7);

    // ── Z8: 底部栏 — 行动顺序 ──
    let z8 = spawn_zone(&mut commands, &theme, BattleZone::Z8BottomBar);
    commands.entity(z8).set_parent_in_place(root);
    let turn_order_bar = spawn_turn_order_bar(&mut commands, &asset_server, &theme);
    commands.entity(turn_order_bar).set_parent_in_place(z8);
}

/// 清除系统：离开战斗时销毁所有战斗屏幕实体
pub fn despawn_battle_screen(mut commands: Commands, query: Query<Entity, With<BattleScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
