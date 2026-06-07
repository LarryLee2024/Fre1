// UI 模块：信息面板、行动菜单、回合提示

use bevy::prelude::*;
use crate::unit::{Unit, Faction, Selected};
use crate::turn::TurnState;

/// 回合提示文本
#[derive(Component)]
pub struct TurnIndicator;

/// 选中单位信息文本
#[derive(Component)]
pub struct UnitInfoPanel;

/// 行动菜单容器
#[derive(Component)]
pub struct ActionMenu;

/// 生成 UI
pub fn spawn_ui(mut commands: Commands) {
    // 回合提示
    commands.spawn((
        TurnIndicator,
        Text::new("第 1 回合 - 玩家阶段"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // 单位信息面板
    commands.spawn((
        UnitInfoPanel,
        Text::new("选择一个单位"),
        TextFont::from_font_size(18.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

/// 更新回合提示
pub fn update_turn_indicator(
    turn_state: Res<TurnState>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
) {
    if turn_state.is_changed() {
        for mut text in &mut query {
            let faction_name = match turn_state.current_faction {
                Faction::Player => "玩家",
                Faction::Enemy => "敌方",
            };
            **text = format!("第 {} 回合 - {}阶段", turn_state.turn_number, faction_name);
        }
    }
}

/// 更新单位信息面板
pub fn update_unit_info(
    selected_units: Query<&Unit, With<Selected>>,
    mut query: Query<&mut Text, (With<UnitInfoPanel>, Without<TurnIndicator>)>,
) {
    for mut text in &mut query {
        if let Ok(unit) = selected_units.single() {
            **text = format!(
                "HP: {}/{}  ATK: {}  DEF: {}  MOV: {}  Range: {}",
                unit.hp, unit.max_hp, unit.atk, unit.def, unit.mov, unit.attack_range
            );
        } else {
            **text = "选择一个单位".to_string();
        }
    }
}
