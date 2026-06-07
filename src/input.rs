// 输入处理模块：点击选择、移动、攻击、行动菜单

use crate::assets::CnFont;
use crate::combat::{manhattan_distance, skill_range};
use crate::combat_event;
use crate::combat_log::CombatLog;
use crate::map::{GameMap, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{TurnPhase, TurnState};
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit, UnitName,
};
use bevy::prelude::*;

/// 攻击目标坐标
#[derive(Resource, Default)]
pub struct AttackTarget {
    pub coord: Option<IVec2>,
}

/// 移动前位置（用于取消时回退）
#[derive(Resource, Default)]
pub struct PrevPosition {
    pub coord: Option<IVec2>,
}

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

/// 处理玩家点击
pub fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    tiles: Query<&Tile>,
    units: Query<(Entity, &Unit, &GridPosition, &Transform)>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    range_positions: Query<&GridPosition, Or<(With<MovableRange>, With<AttackRange>)>>,
    selected_query: Query<Entity, With<Selected>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut attack_target: ResMut<AttackTarget>,
    mut menu_entity: ResMut<ActionMenuEntity>,
) {
    // 只处理玩家回合
    if turn_state.current_faction != Faction::Player {
        return;
    }

    let left_clicked = mouse_button.just_pressed(MouseButton::Left);
    let right_clicked = mouse_button.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    // 右键取消
    if right_clicked {
        match turn_phase.get() {
            TurnPhase::MoveUnit => {
                clear_selection(&mut commands, &selected_query, &range_markers, &highlights);
                attack_target.coord = None;
                next_phase.set(TurnPhase::SelectUnit);
                return;
            }
            TurnPhase::ActionMenu | TurnPhase::SelectTarget => {
                // 由 handle_right_cancel 处理
                return;
            }
            _ => return,
        }
    }

    // 获取相机（用于世界坐标→屏幕坐标转换）
    let Ok((camera, cam_transform)) = camera_query.single() else {
        return;
    };

    // 获取点击的世界坐标
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else {
        return;
    };
    let coord = map.world_to_coord(world_pos);
    if !map.is_in_bounds(coord) {
        return;
    }

    match turn_phase.get() {
        TurnPhase::SelectUnit => {
            // 点击选择己方未行动单位
            for (entity, unit, gp, _) in &units {
                if unit.faction == Faction::Player && !unit.acted && gp.coord == coord {
                    clear_selection(&mut commands, &selected_query, &range_markers, &highlights);
                    commands.entity(entity).insert(Selected);
                    show_move_range(&mut commands, &map, &tiles, &units, unit, gp.coord);
                    spawn_selection_highlight(&mut commands, &map, gp.coord);
                    next_phase.set(TurnPhase::MoveUnit);
                    return;
                }
            }
        }
        TurnPhase::MoveUnit => {
            // 检查是否点击了可移动格子
            let is_movable = range_positions.iter().any(|gp| gp.coord == coord);

            if is_movable {
                if let Ok(selected_entity) = selected_query.single() {
                    // 记录移动前位置
                    if let Ok((_, _, old_gp, _)) = units.get(selected_entity) {
                        commands.insert_resource(PrevPosition {
                            coord: Some(old_gp.coord),
                        });
                    }

                    let world_pos = map.coord_to_world(coord);
                    commands
                        .entity(selected_entity)
                        .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                        .insert(GridPosition { coord });
                    // 更新高亮位置
                    for h in &highlights {
                        commands.entity(h).despawn();
                    }
                    spawn_selection_highlight(&mut commands, &map, coord);
                }
            }

            // 清除移动范围
            for marker in &range_markers {
                commands.entity(marker).despawn();
            }

            // 弹出行动菜单（用屏幕坐标定位）
            if let Ok(selected_entity) = selected_query.single() {
                if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                    let unit_world = map.coord_to_world(gp.coord);
                    if let Ok(screen_pos) =
                        camera.world_to_viewport(cam_transform, unit_world.extend(1.0))
                    {
                        spawn_action_menu(
                            &mut commands,
                            screen_pos.x,
                            screen_pos.y,
                            unit,
                            &mut menu_entity,
                        );
                    }
                }
            }
            next_phase.set(TurnPhase::ActionMenu);
        }
        TurnPhase::SelectTarget => {
            // 点击敌方单位：攻击
            for (_, unit, gp, _) in &units {
                if unit.faction == Faction::Enemy {
                    if let Ok(selected_entity) = selected_query.single() {
                        if let Ok((_, sel_unit, sel_gp, _)) = units.get(selected_entity) {
                            let effective_range =
                                skill_range(&sel_unit.skill, sel_unit.attack_range);
                            if manhattan_distance(sel_gp.coord, gp.coord) <= effective_range {
                                attack_target.coord = Some(gp.coord);
                                next_phase.set(TurnPhase::ExecuteAction);
                                return;
                            }
                        }
                    }
                }
            }

            // 点击其他地方：回到行动菜单
            clear_markers(&mut commands, &range_markers, &highlights);
            if let Ok(selected_entity) = selected_query.single() {
                if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                    let unit_world = map.coord_to_world(gp.coord);
                    if let Ok(screen_pos) =
                        camera.world_to_viewport(cam_transform, unit_world.extend(1.0))
                    {
                        spawn_action_menu(
                            &mut commands,
                            screen_pos.x,
                            screen_pos.y,
                            unit,
                            &mut menu_entity,
                        );
                    }
                }
            }
            next_phase.set(TurnPhase::ActionMenu);
        }
        _ => {}
    }
}

/// 处理右键取消（ActionMenu/SelectTarget 阶段）
pub fn handle_right_cancel(
    mouse_button: Res<ButtonInput<MouseButton>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    map: Res<GameMap>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    units: Query<(Entity, &Unit, &GridPosition, &Transform)>,
    selected_query: Query<Entity, With<Selected>>,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    prev_position: Res<PrevPosition>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    children_query: Query<&Children>,
    menu_buttons: Query<Entity, With<ActionMenuButton>>,
    mut attack_target: ResMut<AttackTarget>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    match turn_phase.get() {
        TurnPhase::ActionMenu => {
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
        TurnPhase::SelectTarget => {
            clear_markers(&mut commands, &range_markers, &highlights);
            attack_target.coord = None;
            // 重新显示行动菜单
            if let Ok(selected_entity) = selected_query.single() {
                if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                    let Ok((camera, cam_transform)) = camera_query.single() else {
                        return;
                    };
                    let unit_world = map.coord_to_world(gp.coord);
                    if let Ok(screen_pos) =
                        camera.world_to_viewport(cam_transform, unit_world.extend(1.0))
                    {
                        spawn_action_menu(
                            &mut commands,
                            screen_pos.x,
                            screen_pos.y,
                            unit,
                            &mut menu_entity,
                        );
                    }
                }
            }
            next_phase.set(TurnPhase::ActionMenu);
        }
        _ => {}
    }
}

/// 处理行动菜单按钮交互
pub fn handle_action_menu_interaction(
    turn_state: Res<TurnState>,
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
    mut attack_target: ResMut<AttackTarget>,
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

        // 关闭菜单
        despawn_action_menu(&mut commands, &mut menu_entity);

        match button.kind {
            ActionKind::Attack => {
                // 显示攻击范围，进入选择目标阶段
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                        let effective_range = skill_range(&unit.skill, unit.attack_range);
                        show_attack_range(&mut commands, &map, gp.coord, effective_range);
                    }
                }
                next_phase.set(TurnPhase::SelectTarget);
            }
            ActionKind::Skill => {
                // 当前技能自动触发，等同于攻击
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                        let effective_range = skill_range(&unit.skill, unit.attack_range);
                        show_attack_range(&mut commands, &map, gp.coord, effective_range);
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
                );
                attack_target.coord = None;
                next_phase.set(TurnPhase::SelectUnit);
            }
        }
        return;
    }
}

/// 生成行动菜单（弹出式，使用屏幕坐标）
fn spawn_action_menu(
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

    // 菜单位置：单位右侧偏移，确保不超出屏幕
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
fn despawn_action_menu(
    commands: &mut Commands,
    menu_entity: &mut ActionMenuEntity,
    children_query: &Query<&Children>,
    menu_buttons: &Query<Entity, With<ActionMenuButton>>,
) {
    if let Some(entity) = menu_entity.entity {
        // 先销毁子实体中的按钮文本等
        if let Ok(children) = children_query.get(entity) {
            for &child in children.iter() {
                if let Ok(grandchildren) = children_query.get(child) {
                    for &gc in grandchildren.iter() {
                        commands.entity(gc).despawn();
                    }
                }
                commands.entity(child).despawn();
            }
        }
        // 再销毁菜单根
        commands.entity(entity).despawn();
        menu_entity.entity = None;
    }
    // 清理可能残留的孤儿按钮
    for btn in menu_buttons {
        commands.entity(btn).despawn();
    }
}

/// 撤销移动（取消时回退到移动前位置）
fn cancel_move(
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

    clear_selection(commands, selected_query, range_markers, highlights);
}

/// 按 E 键结束回合
pub fn handle_end_turn(
    keyboard: Res<ButtonInput<KeyCode>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut units: Query<&mut Unit>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    // 仅在 SelectUnit 阶段允许结束回合
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    // 清除选中状态
    clear_selection(&mut commands, &selected_query, &range_markers, &highlights);

    // 标记所有玩家单位为已行动
    for mut unit in units.iter_mut() {
        if unit.faction == Faction::Player {
            unit.acted = true;
        }
    }

    next_phase.set(TurnPhase::TurnEnd);
}

/// 清除范围标记和高亮（不含 Selected 移除，由调用方处理）
fn clear_markers(
    commands: &mut Commands,
    range_markers: &Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for marker in range_markers {
        commands.entity(marker).despawn();
    }
    for h in highlights {
        commands.entity(h).despawn();
    }
}

/// 清除选中状态和范围标记
fn clear_selection(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    range_markers: &Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for entity in selected_query {
        commands.entity(entity).remove::<Selected>();
    }
    clear_markers(commands, range_markers, highlights);
}

/// 显示可移动范围（蓝色半透明）
fn show_move_range(
    commands: &mut Commands,
    map: &GameMap,
    tiles: &Query<&Tile>,
    units: &Query<(Entity, &Unit, &GridPosition, &Transform)>,
    unit: &Unit,
    start_coord: IVec2,
) {
    let terrain_map = build_tile_terrain_map(tiles);
    let occupation_units: Vec<(IVec2, Faction)> = units
        .iter()
        .map(|(_, u, gp, _)| (gp.coord, u.faction))
        .collect();
    let occupation_map: std::collections::HashMap<IVec2, bool> = occupation_units
        .iter()
        .filter(|(_, f)| *f != unit.faction)
        .map(|(coord, _)| (*coord, true))
        .collect();

    let reachable = find_reachable_tiles(start_coord, unit.mov, map, &terrain_map, &occupation_map);
    let tile_size = map.tile_size;

    for (coord, _) in reachable {
        let world_pos = map.coord_to_world(coord);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.3, 0.6, 1.0, 0.4),
                custom_size: Some(Vec2::splat(tile_size - 2.0)),
                ..default()
            },
            Transform::from_xyz(world_pos.x, world_pos.y, 0.5),
            MovableRange,
            GridPosition { coord },
        ));
    }
}

/// 显示攻击范围（红色半透明）
fn show_attack_range(commands: &mut Commands, map: &GameMap, center: IVec2, range: u32) {
    let tile_size = map.tile_size;
    let range_i32 = range as i32;

    for dx in -range_i32..=range_i32 {
        for dy in -range_i32..=range_i32 {
            if dx.unsigned_abs() + dy.unsigned_abs() > range || (dx == 0 && dy == 0) {
                continue;
            }
            let coord = center + IVec2::new(dx, dy);
            if !map.is_in_bounds(coord) {
                continue;
            }
            let world_pos = map.coord_to_world(coord);
            commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.3, 0.2, 0.35),
                    custom_size: Some(Vec2::splat(tile_size - 2.0)),
                    ..default()
                },
                Transform::from_xyz(world_pos.x, world_pos.y, 0.6),
                AttackRange,
                GridPosition { coord },
            ));
        }
    }
}

/// 生成选中高亮（黄色半透明框）
fn spawn_selection_highlight(commands: &mut Commands, map: &GameMap, coord: IVec2) {
    let world_pos = map.coord_to_world(coord);
    let tile_size = map.tile_size;
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.3, 0.5),
            custom_size: Some(Vec2::splat(tile_size * 0.75)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 0.8),
        SelectionHighlight,
    ));
}

/// 执行攻击（OnEnter）
pub fn execute_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit, &GridPosition, &UnitName), With<Selected>>,
    mut targets: Query<
        (Entity, &mut Unit, &GridPosition, &UnitName, &Transform),
        Without<Selected>,
    >,
    tiles: Query<&Tile>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    mut attack_target: ResMut<AttackTarget>,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut combat_log: ResMut<CombatLog>,
    cn_font: Res<CnFont>,
) {
    // 清除范围标记和高亮
    clear_markers(&mut commands, &range_markers, &highlights);

    if let Ok((entity, mut unit, _pos, attacker_name)) = selected_units.single_mut() {
        // 查找攻击目标
        if let Some(target_coord) = attack_target.coord {
            for (target_entity, mut target, target_pos, target_name, target_transform) in
                targets.iter_mut()
            {
                if target_pos.coord == target_coord && target.faction != unit.faction {
                    let terrain =
                        tiles
                            .iter()
                            .find_map(|t| {
                                if t.coord == target_pos.coord { Some(t.terrain) } else { None }
                            })
                            .unwrap_or(crate::map::Terrain::Plain);

                    // 统一攻击处理
                    combat_event::execute_attack(
                        &mut commands,
                        &unit,
                        &attacker_name.0,
                        target_entity,
                        &mut target,
                        target_pos.coord,
                        target_name,
                        target_transform.translation.truncate(),
                        terrain,
                        &cn_font,
                        &mut combat_log,
                    );
                    break;
                }
            }
        }

        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    attack_target.coord = None;
    next_phase.set(TurnPhase::TurnEnd);
}

/// 待机（OnEnter）
pub fn wait_action_on_enter(
    mut selected_units: Query<(Entity, &mut Unit), With<Selected>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
) {
    // 清除范围标记和高亮
    clear_markers(&mut commands, &range_markers, &highlights);

    if let Ok((entity, mut unit)) = selected_units.single_mut() {
        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    next_phase.set(TurnPhase::TurnEnd);
}

/// 回合结束（OnEnter）
pub fn turn_end_on_enter(
    mut turn_state: ResMut<TurnState>,
    mut units: Query<&mut Unit>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut ai_timer: ResMut<crate::turn::AiTimer>,
) {
    let current_faction = turn_state.current_faction;

    // 检查当前阵营是否所有单位都已行动
    let all_acted = units
        .iter_mut()
        .filter(|u| u.faction == current_faction)
        .all(|u| u.acted);

    if all_acted {
        let next_faction = match current_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => {
                turn_state.turn_number += 1;
                Faction::Player
            }
        };
        turn_state.current_faction = next_faction;

        // 重置新阵营单位的行动状态
        for mut unit in units.iter_mut() {
            if unit.faction == next_faction {
                unit.acted = false;
            }
        }

        // 切换到敌方时重置 AI 计时器
        if next_faction == Faction::Enemy {
            ai_timer.timer.reset();
        }
    }

    next_phase.set(TurnPhase::SelectUnit);
}
