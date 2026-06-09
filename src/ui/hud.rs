// HUD 模块：信息面板、回合提示、HP 条
// UI 只读 ViewModel Resource，不直接 Query 游戏组件

use crate::assets::CnFont;
use crate::battle::CombatLogPanel;
use crate::character::{HpBarFg, Unit};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::turn::{AppState, TurnPhase};
use crate::ui::theme::UiTheme;
use crate::ui::view_models::{CombatPreviewView, GameOverState, SelectedUnitView, TurnInfoView};
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

/// 面板文本标签（统一枚举，避免多个 &mut Text Query 冲突）
#[derive(Component)]
pub enum PanelLabel {
    UnitName,
    Hp,
    Mp,
    Stamina,
    CoreAttrs,
    CombatAttrs,
    SupportAttrs,
    Skills,
}

// ── UI 组件标记 ──

/// 回合提示文本
#[derive(Component)]
pub struct TurnIndicator;

/// 单位信息面板根节点
#[derive(Component)]
pub struct UnitInfoPanel;

/// 行动菜单提示文本
#[derive(Component)]
pub struct ActionMenuText;

/// HP 进度条填充
#[derive(Component)]
pub struct HpBarFill;

/// MP 进度条填充
#[derive(Component)]
pub struct MpBarFill;

/// 耐力进度条填充
#[derive(Component)]
pub struct StaminaBarFill;

/// Buff 容器节点
#[derive(Component)]
pub struct BuffsContainer;

// ── 进度条辅助函数 ──

/// 生成资源条行：标签 + 进度条背景 + 填充 + 数值文本
fn spawn_resource_bar(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    theme: &UiTheme,
    label: &str,
    fill_color: Color,
    fill_marker: impl Component,
    text_label: PanelLabel,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },))
        .with_children(|row| {
            // 标签
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(28.0),
                    ..default()
                },
            ));
            // 进度条背景
            row.spawn((
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            ))
            .with_children(|bar| {
                // 进度条填充
                bar.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                    fill_marker,
                ));
            });
            // 数值文本
            row.spawn((
                Text::new("0/0"),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(Color::WHITE),
                text_label,
            ));
        });
}

// ── UI 生成 ──

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

    // 单位信息面板（卡片式布局）
    let panel_width = 380.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                width: Val::Px(panel_width),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(theme.panel_bg),
            UnitInfoPanel,
        ))
        .with_children(|parent| {
            // 单位名称
            parent.spawn((
                Text::new("选择一个单位"),
                TextFont {
                    font_size: theme.font_medium,
                    ..default()
                },
                TextColor(Color::WHITE),
                PanelLabel::UnitName,
            ));

            // HP 行：标签 + 进度条 + 数值
            spawn_resource_bar(
                parent,
                &theme,
                "HP",
                theme.hp_bar_color,
                HpBarFill,
                PanelLabel::Hp,
            );
            // MP 行
            spawn_resource_bar(
                parent,
                &theme,
                "MP",
                theme.mp_bar_color,
                MpBarFill,
                PanelLabel::Mp,
            );
            // STA 行
            spawn_resource_bar(
                parent,
                &theme,
                "STA",
                theme.stamina_bar_color,
                StaminaBarFill,
                PanelLabel::Stamina,
            );

            // 核心属性
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                PanelLabel::CoreAttrs,
            ));

            // 战斗属性
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_primary),
                PanelLabel::CombatAttrs,
            ));

            // 辅助属性
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_secondary),
                PanelLabel::SupportAttrs,
            ));

            // 技能
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: theme.font_small,
                    ..default()
                },
                TextColor(theme.text_skill),
                PanelLabel::Skills,
            ));

            // Buff 容器
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                BuffsContainer,
            ));
        });

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

// ── UI 更新系统 ──

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
    mut panel_texts: Query<(&PanelLabel, &mut Text)>,
    mut hp_fill_query: Query<&mut Node, With<HpBarFill>>,
    mut mp_fill_query: Query<&mut Node, (With<MpBarFill>, Without<HpBarFill>)>,
    mut stamina_fill_query: Query<
        &mut Node,
        (With<StaminaBarFill>, Without<HpBarFill>, Without<MpBarFill>),
    >,
    mut buffs_container_query: Query<(Entity, &mut Visibility), With<BuffsContainer>>,
    mut commands: Commands,
    theme: Res<UiTheme>,
) {
    if !view.is_changed() {
        return;
    }

    if !view.is_selected {
        // 隐藏面板内容
        for (label, mut text) in &mut panel_texts {
            match label {
                PanelLabel::UnitName => **text = "选择一个单位".to_string(),
                PanelLabel::Hp | PanelLabel::Mp | PanelLabel::Stamina => **text = "0/0".to_string(),
                _ => **text = String::new(),
            }
        }
        for mut node in &mut hp_fill_query {
            node.width = Val::Percent(0.0);
        }
        for mut node in &mut mp_fill_query {
            node.width = Val::Percent(0.0);
        }
        for mut node in &mut stamina_fill_query {
            node.width = Val::Percent(0.0);
        }
        for (container_entity, mut vis) in &mut buffs_container_query {
            *vis = Visibility::Hidden;
            commands.entity(container_entity).despawn_children();
        }
        return;
    }

    // HP 进度条
    let hp_ratio = if view.max_hp > 0 {
        view.hp as f32 / view.max_hp as f32
    } else {
        0.0
    };
    for mut node in &mut hp_fill_query {
        node.width = Val::Percent(hp_ratio * 100.0);
    }

    // MP 进度条
    let mp_ratio = if view.max_mp > 0 {
        view.mp as f32 / view.max_mp as f32
    } else {
        0.0
    };
    for mut node in &mut mp_fill_query {
        node.width = Val::Percent(mp_ratio * 100.0);
    }

    // STA 进度条
    let sta_ratio = if view.max_stamina > 0 {
        view.stamina as f32 / view.max_stamina as f32
    } else {
        0.0
    };
    for mut node in &mut stamina_fill_query {
        node.width = Val::Percent(sta_ratio * 100.0);
    }

    // 核心属性（两行四列）
    let core_line1: Vec<String> = view
        .core_attrs
        .iter()
        .take(4)
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let core_line2: Vec<String> = view
        .core_attrs
        .iter()
        .skip(4)
        .take(4)
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let core_text = format!(
        "── 核心属性 ──\n{}  \n{}",
        core_line1.join("  "),
        core_line2.join("  ")
    );

    // 战斗属性
    let combat_line: Vec<String> = view
        .combat_attrs
        .iter()
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let combat_text = format!("── 战斗属性 ──\n{}", combat_line.join("  "));

    // 辅助属性
    let support_line: Vec<String> = view
        .support_attrs
        .iter()
        .map(|a| format!("{}:{}", a.label, a.value))
        .collect();
    let support_text = format!("── 辅助属性 ──\n{}", support_line.join("  "));

    // 技能
    let skill_names: Vec<String> = view.skills.iter().map(|s| s.name.clone()).collect();
    let skills_text = format!(
        "── 技能 ──\n{}",
        if skill_names.is_empty() {
            "无".to_string()
        } else {
            skill_names.join(" / ")
        }
    );

    // 统一更新所有面板文本
    for (label, mut text) in &mut panel_texts {
        match label {
            PanelLabel::UnitName => **text = view.name.clone(),
            PanelLabel::Hp => **text = format!("{}/{}", view.hp, view.max_hp),
            PanelLabel::Mp => **text = format!("{}/{}", view.mp, view.max_mp),
            PanelLabel::Stamina => **text = format!("{}/{}", view.stamina, view.max_stamina),
            PanelLabel::CoreAttrs => **text = core_text.clone(),
            PanelLabel::CombatAttrs => **text = combat_text.clone(),
            PanelLabel::SupportAttrs => **text = support_text.clone(),
            PanelLabel::Skills => **text = skills_text.clone(),
        }
    }

    // Buff 彩色标签
    for (container_entity, mut vis) in &mut buffs_container_query {
        if view.buffs.is_empty() {
            *vis = Visibility::Hidden;
            commands.entity(container_entity).despawn_children();
        } else {
            *vis = Visibility::Visible;
            commands.entity(container_entity).despawn_children();
            commands.entity(container_entity).with_children(|parent| {
                parent.spawn((
                    Text::new("── 状态 ── "),
                    TextFont {
                        font_size: theme.font_small,
                        ..default()
                    },
                    TextColor(theme.text_secondary),
                ));
                for buff in &view.buffs {
                    let color = if buff.is_buff {
                        theme.buff_color
                    } else {
                        theme.debuff_color
                    };
                    parent.spawn((
                        Text::new(format!("[{}·{}t]", buff.name, buff.remaining_turns)),
                        TextFont {
                            font_size: theme.font_small,
                            ..default()
                        },
                        TextColor(color),
                    ));
                }
            });
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

/// 更新 HP 条宽度（地图上单位的 HP 条）
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
            .init_resource::<CombatPreviewView>()
            .add_systems(OnEnter(AppState::InGame), spawn_ui.in_set(GameSet::Ui))
            .add_systems(
                Update,
                (
                    setup_ui_font,
                    crate::ui::view_models::update_selected_unit_view,
                    crate::ui::view_models::update_turn_info_view,
                    crate::ui::view_models::update_game_over_state,
                    crate::ui::view_models::update_combat_preview_view,
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
