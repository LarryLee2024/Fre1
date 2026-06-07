// 输入处理模块：点击选择、移动、攻击分发

use crate::action_menu::{ActionMenuEntity, cancel_move, spawn_action_menu};
use crate::combat::{manhattan_distance, skill_range};
use crate::map::{GameMap, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{TurnPhase, TurnState};
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit,
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

    // 右键处理
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
            _ => {}
        }
    }

    // 获取点击坐标
    let (camera, cam_transform) = match camera_query.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let coord = match cursor_to_coord(&windows, camera, cam_transform, &map) {
        Some(c) => c,
        None => return,
    };

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
                        commands.entity(h).try_despawn();
                    }
                    spawn_selection_highlight(&mut commands, &map, coord);
                }
            }

            // 清除移动范围
            for marker in &range_markers {
                commands.entity(marker).try_despawn();
            }

            // 弹出行动菜单
            show_action_menu_at_selected(
                &mut commands,
                &units,
                &selected_query,
                &map,
                camera,
                cam_transform,
                &mut menu_entity,
            );
            next_phase.set(TurnPhase::ActionMenu);
        }
        TurnPhase::SelectTarget => {
            // 查找点击坐标处的敌方单位
            let mut clicked_enemy: Option<IVec2> = None;
            for (_, unit, gp, _) in &units {
                if unit.faction == Faction::Enemy && gp.coord == coord {
                    clicked_enemy = Some(gp.coord);
                    break;
                }
            }

            if let Some(enemy_coord) = clicked_enemy {
                // 检查是否在攻击范围内
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, sel_unit, sel_gp, _)) = units.get(selected_entity) {
                        let effective_range = skill_range(&sel_unit.skill, sel_unit.attack_range);
                        if manhattan_distance(sel_gp.coord, enemy_coord) <= effective_range {
                            attack_target.coord = Some(enemy_coord);
                            next_phase.set(TurnPhase::ExecuteAction);
                            return;
                        }
                    }
                }
            }

            // 点击其他地方或超出范围：回到行动菜单
            clear_markers(&mut commands, &range_markers, &highlights);
            show_action_menu_at_selected(
                &mut commands,
                &units,
                &selected_query,
                &map,
                camera,
                cam_transform,
                &mut menu_entity,
            );
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
    menu_buttons: Query<Entity, With<crate::action_menu::ActionMenuButton>>,
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
            if let Ok((camera, cam_transform)) = camera_query.single() {
                show_action_menu_at_selected(
                    &mut commands,
                    &units,
                    &selected_query,
                    &map,
                    camera,
                    cam_transform,
                    &mut menu_entity,
                );
            }
            next_phase.set(TurnPhase::ActionMenu);
        }
        _ => {}
    }
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

// ── 公共辅助函数 ──

/// 屏幕光标 → 格子坐标
pub fn cursor_to_coord(
    windows: &Query<&Window>,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    map: &GameMap,
) -> Option<IVec2> {
    let Ok(window) = windows.single() else { return None };
    let cursor_pos = window.cursor_position()?;
    let world_pos = camera
        .viewport_to_world_2d(cam_transform, cursor_pos)
        .ok()?;
    let coord = map.world_to_coord(world_pos);
    map.is_in_bounds(coord).then_some(coord)
}

/// 在选中单位位置弹出行动菜单（消除重复代码）
pub fn show_action_menu_at_selected(
    commands: &mut Commands,
    units: &Query<(Entity, &Unit, &GridPosition, &Transform)>,
    selected_query: &Query<Entity, With<Selected>>,
    map: &GameMap,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    menu_entity: &mut ActionMenuEntity,
) {
    if let Ok(selected_entity) = selected_query.single() {
        if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
            let unit_world = map.coord_to_world(gp.coord);
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_world.extend(1.0))
            {
                spawn_action_menu(commands, screen_pos.x, screen_pos.y, unit, menu_entity);
            }
        }
    }
}

/// 清除范围标记和高亮（不含 Selected 移除，由调用方处理）
pub fn clear_markers(
    commands: &mut Commands,
    range_markers: &Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for marker in range_markers {
        commands.entity(marker).try_despawn();
    }
    for h in highlights {
        commands.entity(h).try_despawn();
    }
}

/// 清除选中状态和范围标记
pub fn clear_selection(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    range_markers: &Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for entity in selected_query {
        commands.entity(entity).remove::<Selected>();
    }
    clear_markers(commands, range_markers, &highlights);
}

/// 显示可移动范围（蓝色半透明）
pub fn show_move_range(
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
pub fn show_attack_range(commands: &mut Commands, map: &GameMap, center: IVec2, range: u32) {
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
pub fn spawn_selection_highlight(commands: &mut Commands, map: &GameMap, coord: IVec2) {
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
