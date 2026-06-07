// UI 模块：信息面板、行动菜单、回合提示、HP 条

use crate::turn::{AppState, TurnPhase, TurnState};
use crate::unit::{Faction, HpBarFg, Selected, Unit};
use bevy::prelude::*;

/// 中文字体路径
const CN_FONT: &str = "fonts/Arial Unicode.ttf";

/// 回合提示文本
#[derive(Component)]
pub struct TurnIndicator;

/// 选中单位信息文本
#[derive(Component)]
pub struct UnitInfoPanel;

/// 行动菜单提示文本
#[derive(Component)]
pub struct ActionMenuText;

/// 生成 UI
pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(CN_FONT);

    // 回合提示
    commands.spawn((
        TurnIndicator,
        Text::new("第 1 回合 - 玩家阶段"),
        TextFont {
            font: font.clone(),
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // 操作提示
    commands.spawn((
        Text::new("左键选择/移动 | 右键取消 | E 结束回合"),
        TextFont {
            font: font.clone(),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // 单位信息面板
    commands.spawn((
        UnitInfoPanel,
        Text::new("选择一个单位"),
        TextFont {
            font: font.clone(),
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // 行动菜单提示（默认隐藏）
    commands.spawn((
        ActionMenuText,
        Text::new("点击敌方单位攻击 | 右键/点击空地待机"),
        TextFont {
            font,
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        Visibility::Hidden,
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
    selected_units: Query<(&Unit, &crate::unit::UnitName), With<Selected>>,
    mut query: Query<
        &mut Text,
        (
            With<UnitInfoPanel>,
            Without<TurnIndicator>,
            Without<ActionMenuText>,
        ),
    >,
) {
    for mut text in &mut query {
        if let Ok((unit, name)) = selected_units.single() {
            **text = format!(
                "{}  HP: {}/{}  ATK: {}  DEF: {}  MOV: {}  Range: {}",
                name.0, unit.hp, unit.max_hp, unit.atk, unit.def, unit.mov, unit.attack_range
            );
        } else {
            **text = "选择一个单位".to_string();
        }
    }
}

/// 更新行动菜单可见性
pub fn update_action_menu(
    turn_phase: Res<State<TurnPhase>>,
    turn_state: Res<TurnState>,
    mut query: Query<&mut Visibility, With<ActionMenuText>>,
) {
    for mut vis in &mut query {
        *vis = if *turn_phase.get() == TurnPhase::SelectAction
            && turn_state.current_faction == Faction::Player
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 更新 HP 条宽度
pub fn update_hp_bars(
    units: Query<(&Unit, &Children), With<Unit>>,
    mut hp_fgs: Query<&mut Sprite, With<HpBarFg>>,
    map: Res<crate::map::GameMap>,
) {
    let bar_width = map.tile_size * 0.6;
    for (unit, children) in &units {
        let ratio = (unit.hp as f32 / unit.max_hp as f32).max(0.0);
        for child in children.iter() {
            if let Ok(mut sprite) = hp_fgs.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(bar_width * ratio, 4.0));
            }
        }
    }
}

/// 检查胜负条件
pub fn check_game_over(
    units: Query<&Unit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut turn_indicator: Query<&mut Text, With<TurnIndicator>>,
) {
    let has_player = units.iter().any(|u| u.faction == Faction::Player);
    let has_enemy = units.iter().any(|u| u.faction == Faction::Enemy);

    if !has_enemy {
        for mut text in &mut turn_indicator {
            **text = "胜利！".to_string();
        }
        next_app_state.set(AppState::GameOver);
    } else if !has_player {
        for mut text in &mut turn_indicator {
            **text = "失败...".to_string();
        }
        next_app_state.set(AppState::GameOver);
    }
}
