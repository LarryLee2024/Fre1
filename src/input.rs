// 输入处理模块：点击选择、移动、攻击分发
// 使用 Attributes/SkillSlots 替代原 Unit 上的硬编码属性

use crate::action_menu::{ActionMenuEntity, cancel_move, spawn_action_menu};
use crate::combat::manhattan_distance;
use crate::combat_event::{CombatIntent, PrevPosition};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::data::skill_data::{effective_skill_range, SkillRegistry, SkillSlots};
use crate::map::{GameMap, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{TurnPhase, TurnState};
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit,
};
use bevy::prelude::*;

/// 处理玩家点击
pub fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    tiles: Query<&Tile>,
    units: Query<(Entity, &Unit, &GridPosition, &Transform, &Attributes, &SkillSlots)>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    // 合并 range_markers + range_positions 为一个查询，减少参数数量
    range_entities: Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    selected_query: Query<Entity, With<Selected>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut combat_intent: ResMut<CombatIntent>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    skill_registry: Res<SkillRegistry>,
) {
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
                clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                combat_intent.target_coord = None;
                next_phase.set(TurnPhase::SelectUnit);
                return;
            }
            TurnPhase::ActionMenu | TurnPhase::SelectTarget => {
                return;
            }
            _ => {}
        }
    }

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
            for (entity, unit, gp, _, _, _) in &units {
                if unit.faction == Faction::Player && !unit.acted && gp.coord == coord {
                    clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                    commands.entity(entity).insert(Selected);
                    show_move_range(&mut commands, &map, &tiles, &units, unit, gp.coord);
                    spawn_selection_highlight(&mut commands, &map, gp.coord);
                    next_phase.set(TurnPhase::MoveUnit);
                    return;
                }
            }
        }
        TurnPhase::MoveUnit => {
            let is_movable = range_entities.iter().any(|(_, gp)| gp.map(|g| g.coord == coord).unwrap_or(false));

            if is_movable {
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, _, old_gp, _, _, _)) = units.get(selected_entity) {
                        commands.insert_resource(PrevPosition {
                            coord: Some(old_gp.coord),
                        });
                    }

                    let world_pos = map.coord_to_world(coord);
                    commands
                        .entity(selected_entity)
                        .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                        .insert(GridPosition { coord });
                    for h in &highlights {
                        commands.entity(h).try_despawn();
                    }
                    spawn_selection_highlight(&mut commands, &map, coord);
                }
            }

            for (marker, _) in &range_entities {
                commands.entity(marker).try_despawn();
            }

            show_action_menu_at_selected(
                &mut commands,
                &units,
                &selected_query,
                &map,
                camera,
                cam_transform,
                &mut menu_entity,
                &skill_registry,
            );
            next_phase.set(TurnPhase::ActionMenu);
        }
        TurnPhase::SelectTarget => {
            let mut clicked_enemy: Option<IVec2> = None;
            for (_, unit, gp, _, _, _) in &units {
                if unit.faction == Faction::Enemy && gp.coord == coord {
                    clicked_enemy = Some(gp.coord);
                    break;
                }
            }

            if let Some(enemy_coord) = clicked_enemy {
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, _, sel_gp, _, _, _sel_skills)) = units.get(selected_entity) {
                        let skill_id = combat_intent
                            .skill_id
                            .as_deref()
                            .unwrap_or("basic_attack");
                        if let Some(skill_data) = skill_registry.get(skill_id) {
                            let base_range = units
                                .get(selected_entity)
                                .map(|(_, _, _, _, attrs, _)| attrs.get(AttributeKind::AttackRange) as u32)
                                .unwrap_or(1);
                            let effective_range = effective_skill_range(skill_data, base_range);
                            if manhattan_distance(sel_gp.coord, enemy_coord) <= effective_range {
                                combat_intent.target_coord = Some(enemy_coord);
                                next_phase.set(TurnPhase::ExecuteAction);
                                return;
                            }
                        }
                    }
                }
            }

            clear_markers(&mut commands, &range_entities, &highlights);
            show_action_menu_at_selected(
                &mut commands,
                &units,
                &selected_query,
                &map,
                camera,
                cam_transform,
                &mut menu_entity,
                &skill_registry,
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
    units: Query<(Entity, &Unit, &GridPosition, &Transform, &Attributes, &SkillSlots)>,
    selected_query: Query<Entity, With<Selected>>,
    range_entities: Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    prev_position: Res<PrevPosition>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    mut combat_intent: ResMut<CombatIntent>,
    skill_registry: Res<SkillRegistry>,
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
                &range_entities,
                &highlights,
                &prev_position,
                &map,
                &mut menu_entity,
            );
            combat_intent.target_coord = None;
            combat_intent.skill_id = None;
            next_phase.set(TurnPhase::SelectUnit);
        }
        TurnPhase::SelectTarget => {
            clear_markers(&mut commands, &range_entities, &highlights);
            combat_intent.target_coord = None;
            combat_intent.skill_id = None;
            if let Ok((camera, cam_transform)) = camera_query.single() {
                show_action_menu_at_selected(
                    &mut commands,
                    &units,
                    &selected_query,
                    &map,
                    camera,
                    cam_transform,
                    &mut menu_entity,
                    &skill_registry,
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
    range_entities: Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    clear_selection(&mut commands, &selected_query, &range_entities, &highlights);

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
    let Ok(window) = windows.single() else {
        return None;
    };
    let cursor_pos = window.cursor_position()?;
    let world_pos = camera
        .viewport_to_world_2d(cam_transform, cursor_pos)
        .ok()?;
    let coord = map.world_to_coord(world_pos);
    map.is_in_bounds(coord).then_some(coord)
}

/// 在选中单位位置弹出行动菜单
pub fn show_action_menu_at_selected(
    commands: &mut Commands,
    units: &Query<(Entity, &Unit, &GridPosition, &Transform, &Attributes, &SkillSlots)>,
    selected_query: &Query<Entity, With<Selected>>,
    map: &GameMap,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    menu_entity: &mut ActionMenuEntity,
    skill_registry: &SkillRegistry,
) {
    if let Ok(selected_entity) = selected_query.single() {
        if let Ok((_, unit, gp, _, _, skill_slots)) = units.get(selected_entity) {
            let unit_world = map.coord_to_world(gp.coord);
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_world.extend(1.0)) {
                spawn_action_menu(
                    commands,
                    screen_pos.x,
                    screen_pos.y,
                    unit,
                    skill_slots,
                    menu_entity,
                    skill_registry,
                );
            }
        }
    }
}

/// 清除范围标记和高亮
pub fn clear_markers(
    commands: &mut Commands,
    range_entities: &Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for (marker, _) in range_entities {
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
    range_entities: &Query<(Entity, Option<&GridPosition>), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for entity in selected_query {
        commands.entity(entity).remove::<Selected>();
    }
    clear_markers(commands, range_entities, highlights);
}

/// 显示可移动范围
pub fn show_move_range(
    commands: &mut Commands,
    map: &GameMap,
    tiles: &Query<&Tile>,
    units: &Query<(Entity, &Unit, &GridPosition, &Transform, &Attributes, &SkillSlots)>,
    unit: &Unit,
    start_coord: IVec2,
) {
    let terrain_map = build_tile_terrain_map(tiles);
    let occupation_units: Vec<(IVec2, Faction)> = units
        .iter()
        .map(|(_, u, gp, _, _, _)| (gp.coord, u.faction))
        .collect();
    let occupation_map: std::collections::HashMap<IVec2, bool> = occupation_units
        .iter()
        .filter(|(_, f)| *f != unit.faction)
        .map(|(coord, _)| (*coord, true))
        .collect();

    // 从 Attributes 获取移动力
    let move_points = units
        .iter()
        .find(|(_, u, gp, _, _, _)| u.faction == unit.faction && gp.coord == start_coord)
        .map(|(_, _, _, _, attrs, _)| attrs.get(AttributeKind::Mov) as u32)
        .unwrap_or(3);

    let reachable = find_reachable_tiles(start_coord, move_points, map, &terrain_map, &occupation_map);
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

/// 显示攻击范围
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

/// 生成选中高亮
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

/// 输入处理插件
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.add_systems(
            Update,
            handle_click.run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            handle_right_cancel.run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            handle_end_turn.run_if(in_state(AppState::InGame)),
        );
    }
}
