// HUD 模块：信息面板、回合提示、HP 条
// UI 只读 ViewModel Resource，不直接 Query 游戏组件

use crate::assets::CnFont;
use crate::battle::CombatLogPanel;
use crate::character::{HpBarFg, Unit};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::turn::{AppState, TurnPhase};
use crate::ui::theme::UiTheme;
use crate::ui::view_models::{GameOverState, SelectedUnitView, TurnInfoView};
use bevy::prelude::*;

// ── UI 组件标记 ──

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
pub fn spawn_ui(mut commands: Commands, theme: Res<UiTheme>) {
    // 回合提示
    commands.spawn((
        Text::new("第 1 回合 - 玩家阶段"),
        TextFont {
            font_size: theme.font_large,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        TurnIndicator,
    ));

    // 操作提示
    commands.spawn((
        Text::new("左键选择/移动 | 右键取消/菜单 | E 结束回合"),
        TextFont {
            font_size: theme.font_small,
            ..default()
        },
        TextColor(theme.text_secondary),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // 单位信息面板
    commands.spawn((
        Text::new("选择一个单位"),
        TextFont {
            font_size: theme.font_medium,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        UnitInfoPanel,
    ));

    // 行动菜单提示（默认隐藏）
    commands.spawn((
        Text::new("选择行动：攻击/技能/待机/取消"),
        TextFont {
            font_size: theme.font_menu,
            ..default()
        },
        TextColor(theme.text_skill),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        Visibility::Hidden,
        ActionMenuText,
    ));

    // 战斗日志面板（右侧）
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: theme.font_log,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(240.0),
            ..default()
        },
        CombatLogPanel,
    ));
}

/// 设置中文字体
pub fn setup_ui_font(cn_font: Res<CnFont>, mut query: Query<&mut TextFont, With<Node>>) {
    for mut text_font in &mut query {
        text_font.font = cn_font.handle.clone();
    }
}

/// 更新回合提示（读取 TurnInfoView）
pub fn update_turn_indicator(
    turn_view: Res<TurnInfoView>,
    mut query: Query<&mut Text, With<TurnIndicator>>,
) {
    if turn_view.is_changed() {
        for mut text in &mut query {
            **text = format!(
                "第 {} 回合 - {}阶段",
                turn_view.turn_number, turn_view.faction_label
            );
        }
    }
}

/// 更新单位信息面板（读取 SelectedUnitView）
pub fn update_unit_info(
    view: Res<SelectedUnitView>,
    mut query: Query<&mut Text, With<UnitInfoPanel>>,
) {
    if view.is_changed() {
        for mut text in &mut query {
            if view.is_selected {
                **text = format!(
                    "{}  HP: {}/{}  ATK: {}  DEF: {}  MAG: {}  MDEF: {}  MOV: {}\n{}  技能: {}\n状态: {}",
                    view.name,
                    view.hp,
                    view.max_hp,
                    view.atk,
                    view.def,
                    view.magic_attack,
                    view.magic_defense,
                    view.mov,
                    view.core_attrs,
                    view.skills,
                    view.buffs
                );
            } else {
                **text = "选择一个单位".to_string();
            }
        }
    }
}

/// 更新行动菜单可见性
pub fn update_action_menu(
    turn_phase: Res<State<TurnPhase>>,
    turn_view: Res<TurnInfoView>,
    mut query: Query<&mut Visibility, With<ActionMenuText>>,
) {
    for mut vis in &mut query {
        *vis = if *turn_phase.get() == TurnPhase::ActionMenu && turn_view.is_player_turn {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 更新 HP 条宽度
pub fn update_hp_bars(
    units: Query<(&Attributes, &Children), With<Unit>>,
    mut hp_fgs: Query<&mut Sprite, With<HpBarFg>>,
    map: Res<crate::map::GameMap>,
) {
    let bar_width = map.tile_size * 0.6;
    for (attrs, children) in &units {
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        let ratio = if max_hp > 0.0 {
            (hp / max_hp).max(0.0)
        } else {
            0.0
        };
        for child in children.iter() {
            if let Ok(mut sprite) = hp_fgs.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(bar_width * ratio, 4.0));
            }
        }
    }
}

/// 检查胜负条件（读取 GameOverState ViewModel）
pub fn check_game_over(
    game_over: Res<GameOverState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut turn_indicator: Query<&mut Text, With<TurnIndicator>>,
) {
    if game_over.is_changed() {
        match *game_over {
            GameOverState::Victory => {
                for mut text in &mut turn_indicator {
                    **text = "胜利！".to_string();
                }
                next_app_state.set(AppState::GameOver);
            }
            GameOverState::Defeat => {
                for mut text in &mut turn_indicator {
                    **text = "失败...".to_string();
                }
                next_app_state.set(AppState::GameOver);
            }
            GameOverState::Playing => {}
        }
    }
}

/// HUD 插件
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::GameSet;
        app.init_resource::<SelectedUnitView>()
            .init_resource::<TurnInfoView>()
            .init_resource::<GameOverState>()
            .add_systems(OnEnter(AppState::InGame), spawn_ui.in_set(GameSet::Ui))
            .add_systems(
                Update,
                (
                    setup_ui_font,
                    // ViewModel 更新系统
                    crate::ui::view_models::update_selected_unit_view,
                    crate::ui::view_models::update_turn_info_view,
                    crate::ui::view_models::update_game_over_state,
                    // UI 显示系统（只读 ViewModel）
                    update_turn_indicator,
                    update_unit_info,
                    update_action_menu,
                    update_hp_bars,
                    check_game_over,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
