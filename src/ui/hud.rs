// HUD 模块：信息面板、回合提示、HP 条
// 使用 Attributes/ActiveBuffs/SkillSlots 替代原 Unit 上的硬编码属性

use crate::assets::CnFont;
use crate::battle::CombatLogPanel;
use crate::core::attribute::{AttributeKind, Attributes};
use crate::buff::ActiveBuffs;
use crate::skill::{SkillRegistry, SkillSlots};
use crate::turn::{AppState, TurnPhase, TurnState};
use crate::character::{Faction, HpBarFg, Selected, Unit};
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
pub fn spawn_ui(mut commands: Commands) {
    // 回合提示
    commands.spawn((
        Text::new("第 1 回合 - 玩家阶段"),
        TextFont {
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
        TurnIndicator,
    ));

    // 操作提示
    commands.spawn((
        Text::new("左键选择/移动 | 右键取消/菜单 | E 结束回合"),
        TextFont {
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
        Text::new("选择一个单位"),
        TextFont {
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
        UnitInfoPanel,
    ));

    // 行动菜单提示（默认隐藏）
    commands.spawn((
        Text::new("选择行动：攻击/技能/待机/取消"),
        TextFont {
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
        ActionMenuText,
    ));

    // 战斗日志面板（右侧）
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 13.0,
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
    selected_units: Query<
        (&Unit, &crate::character::UnitName, &Attributes, &SkillSlots, &ActiveBuffs),
        With<Selected>,
    >,
    mut query: Query<
        &mut Text,
        (
            With<UnitInfoPanel>,
            Without<TurnIndicator>,
            Without<ActionMenuText>,
        ),
    >,
    skill_registry: Res<SkillRegistry>,
) {
    for mut text in &mut query {
        if let Ok((_unit, name, attrs, skill_slots, buffs)) = selected_units.single() {
            let hp = attrs.get(AttributeKind::Hp) as i32;
            let max_hp = attrs.get(AttributeKind::MaxHp) as i32;
            let atk = attrs.get(AttributeKind::Atk) as i32;
            let def = attrs.get(AttributeKind::Def) as i32;
            let mov = attrs.get(AttributeKind::Mov) as i32;

            // 技能名称列表
            let skill_names: Vec<String> = skill_slots
                .skill_ids
                .iter()
                .filter_map(|id| skill_registry.get(id).map(|sd| sd.name.clone()))
                .collect();
            let skill_label = if skill_names.is_empty() {
                "无".to_string()
            } else {
                skill_names.join("/")
            };

            // Buff 列表
            let status_text = if buffs.is_empty() {
                "无".to_string()
            } else {
                buffs
                    .iter()
                    .map(|inst| format!("[{}·{}t]", inst.name, inst.remaining_turns))
                    .collect::<Vec<_>>()
                    .join("")
            };

            **text = format!(
                "{}  HP: {}/{}  ATK: {}  DEF: {}  MOV: {}  技能: {}\n状态: {}",
                name.0, hp, max_hp, atk, def, mov, skill_label, status_text
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
        *vis = if *turn_phase.get() == TurnPhase::ActionMenu
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
    units: Query<(&Attributes, &Children), With<Unit>>,
    mut hp_fgs: Query<&mut Sprite, With<HpBarFg>>,
    map: Res<crate::map::GameMap>,
) {
    let bar_width = map.tile_size * 0.6;
    for (attrs, children) in &units {
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        let ratio = if max_hp > 0.0 { (hp / max_hp).max(0.0) } else { 0.0 };
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

/// HUD 插件
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::GameSet;
        app.add_systems(OnEnter(AppState::InGame), spawn_ui.in_set(GameSet::Ui))
            .add_systems(
                Update,
                (
                    setup_ui_font,
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
