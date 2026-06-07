// 行动菜单模块：弹出式菜单的生成、交互、销毁

use crate::combat::skill_range;
use crate::input::PrevPosition;
use crate::map::GameMap;
use crate::turn::TurnPhase;
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected,
    SelectionHighlight, Unit,
};
use bevy::prelude::*;

/// 行动菜单选项
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionKind {
    /// 攻击
    Attack,
    /// 技能
    Skill,
    /// 待机
    Wait,
    /// 取消（撤销移动）
    Cancel,
}

/// 行动菜单标记组件
#[derive(Component)]
pub struct ActionMenuRoot;

/// 行动菜单按钮标记
#[derive(Component)]
pub struct ActionMenuButton {
    pub kind: ActionKind,
}

/// 行动菜单实体追踪（防止重复 despawn）
#[derive(Resource, Default)]
pub struct ActionMenuEntity {
    pub entity: Option<Entity>,
}

/// 处理行动菜单按钮交互
pub fn handle_action_menu_interaction(
    turn_state: Res<crate::turn::TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    map: Res<GameMap>,
    selected_query: Query<Entity, With<Selected>>,
    units: Query<(Entity, &Unit, &GridPosition, &Transform)>,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut action_buttons: Query<(&ActionMenuButton, &Interaction), Changed<Interaction>>,
    prev_position: Res<PrevPosition>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    children_query: Query<&Children>,
    menu_buttons: Query<Entity, With<ActionMenuButton>>,
    mut attack_target: ResMut<crate::input::AttackTarget>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if *turn_phase.get() != TurnPhase::ActionMenu {
        return;
    }

    for (button, interaction) in &mut action_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 关闭菜单（含子实体）
        despawn_action_menu(&mut commands, &mut menu_entity, &children_query, &menu_buttons);

        match button.kind {
            ActionKind::Attack => {
                // 显示攻击范围，进入选择目标阶段
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                        let effective_range = skill_range(&unit.skill, unit.attack_range);
                        crate::input::show_attack_range(
                            &mut commands,
                            &map,
                            gp.coord,
                            effective_range,
                        );
                    }
                }
                next_phase.set(TurnPhase::SelectTarget);
            }
            ActionKind::Skill => {
                // 当前技能自动触发，等同于攻击
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                        let effective_range = skill_range(&unit.skill, unit.attack_range);
                        crate::input::show_attack_range(
                            &mut commands,
                            &map,
                            gp.coord,
                            effective_range,
                        );
                    }
                }
                next_phase.set(TurnPhase::SelectTarget);
            }
            ActionKind::Wait => {
                next_phase.set(TurnPhase::WaitAction);
            }
            ActionKind::Cancel => {
                cancel_move(
                    &mut commands,
                    &selected_query,
                    &range_markers,
                    &highlights,
                    &prev_position,
                    &map,
                    &mut menu_entity,
                    &children_query,
                    &menu_buttons,
                );
                attack_target.coord = None;
                next_phase.set(TurnPhase::SelectUnit);
            }
        }
        return;
    }
}

/// 生成行动菜单（弹出式，使用屏幕坐标）
pub fn spawn_action_menu(
    commands: &mut Commands,
    screen_x: f32,
    screen_y: f32,
    unit: &Unit,
    menu_entity_res: &mut ActionMenuEntity,
) {
    // 构建菜单选项
    let mut items: Vec<(ActionKind, &str, Color)> = vec![
        (ActionKind::Attack, "攻击", Color::srgb(1.0, 0.4, 0.3)),
        (ActionKind::Wait, "待机", Color::srgb(0.6, 0.8, 1.0)),
        (ActionKind::Cancel, "取消", Color::srgb(0.7, 0.7, 0.7)),
    ];

    // 有技能时插入技能选项
    if unit.skill != crate::unit::Skill::None {
        let skill_label = crate::combat::skill_name(&unit.skill);
        items.insert(1, (ActionKind::Skill, skill_label, Color::srgb(1.0, 0.8, 0.3)));
    }

    let button_height = 28.0;
    let button_width = 72.0;

    // 菜单位置：单位右侧偏移
    let menu_x = screen_x + 30.0;
    let menu_y = screen_y - 20.0;

    // 菜单容器
    let menu_id = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(menu_x),
                top: Val::Px(menu_y),
                width: Val::Px(button_width + 16.0),
                height: Val::Auto,
                row_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
        ))
        .insert(ActionMenuRoot)
        .id();

    for (kind, label, color) in items {
        commands.entity(menu_id).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(button_height),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    Button,
                    ActionMenuButton { kind },
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(color),
                    ));
                });
        });
    }

    // 记录菜单实体
    menu_entity_res.entity = Some(menu_id);
}

/// 安全销毁行动菜单（含所有子实体）
pub fn despawn_action_menu(
    commands: &mut Commands,
    menu_entity: &mut ActionMenuEntity,
    children_query: &Query<&Children>,
    menu_buttons: &Query<Entity, With<ActionMenuButton>>,
) {
    if let Some(entity) = menu_entity.entity {
        // 先销毁子实体中的按钮文本等
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(grandchildren) = children_query.get(child) {
                    for gc in grandchildren.iter() {
                        commands.entity(gc).try_despawn();
                    }
                }
                commands.entity(child).try_despawn();
            }
        }
        // 再销毁菜单根
        commands.entity(entity).try_despawn();
        menu_entity.entity = None;
    }
    // 清理可能残留的孤儿按钮
    for btn in menu_buttons {
        commands.entity(btn).try_despawn();
    }
}

/// 撤销移动（取消时回退到移动前位置）
pub fn cancel_move(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    range_markers: &Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
    prev_position: &PrevPosition,
    map: &GameMap,
    menu_entity: &mut ActionMenuEntity,
    children_query: &Query<&Children>,
    menu_buttons: &Query<Entity, With<ActionMenuButton>>,
) {
    // 关闭菜单
    despawn_action_menu(commands, menu_entity, children_query, menu_buttons);

    // 回退位置
    if let Some(prev_coord) = prev_position.coord {
        if let Ok(selected_entity) = selected_query.single() {
            let world_pos = map.coord_to_world(prev_coord);
            commands
                .entity(selected_entity)
                .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                .insert(GridPosition { coord: prev_coord });
        }
    }

    crate::input::clear_selection(commands, selected_query, range_markers, highlights);
}

/// 行动菜单插件
pub struct ActionMenuPlugin;

impl Plugin for ActionMenuPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.init_resource::<ActionMenuEntity>().add_systems(
            Update,
            handle_action_menu_interaction.run_if(in_state(AppState::InGame)),
        );
    }
}
