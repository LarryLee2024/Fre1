// 输入处理模块：点击选择、移动、攻击

use crate::combat::{calculate_damage, manhattan_distance, skill_name, skill_range};
use crate::map::{GameMap, Terrain, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{TurnPhase, TurnState};
use crate::ui::{CombatLog, LogSegment, log_color};
use crate::unit::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Skill, Unit, UnitName,
};
use crate::vfx;
use bevy::prelude::*;

/// 攻击目标坐标
#[derive(Resource, Default)]
pub struct AttackTarget {
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
    range_markers: Query<(Entity, &GridPosition), Or<(With<MovableRange>, With<AttackRange>)>>,
    selected_query: Query<Entity, With<Selected>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut attack_target: ResMut<AttackTarget>,
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

    // 右键取消：回到选择单位阶段
    if right_clicked {
        match turn_phase.get() {
            TurnPhase::MoveUnit | TurnPhase::SelectAction => {
                clear_selection(&mut commands, &selected_query, &range_markers, &highlights);
                attack_target.coord = None;
                next_phase.set(TurnPhase::SelectUnit);
                return;
            }
            _ => return,
        }
    }

    // 获取点击的世界坐标
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_query.single() else {
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
            let is_movable = range_markers.iter().any(|(_, gp)| gp.coord == coord);

            if is_movable {
                if let Ok(selected_entity) = selected_query.single() {
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

            // 清除移动范围，显示攻击范围
            for (marker, _) in &range_markers {
                commands.entity(marker).despawn();
            }

            if let Ok(selected_entity) = selected_query.single() {
                if let Ok((_, unit, gp, _)) = units.get(selected_entity) {
                    let effective_range = skill_range(&unit.skill, unit.attack_range);
                    show_attack_range(&mut commands, &map, gp.coord, effective_range);
                }
            }
            next_phase.set(TurnPhase::SelectAction);
        }
        TurnPhase::SelectAction => {
            // 点击敌方单位：攻击
            for (_, unit, gp, _) in &units {
                if unit.faction == Faction::Enemy {
                    if let Ok(selected_entity) = selected_query.single() {
                        if let Ok((_, sel_unit, sel_gp, _)) = units.get(selected_entity) {
                            let effective_range = skill_range(&sel_unit.skill, sel_unit.attack_range);
                            if manhattan_distance(sel_gp.coord, gp.coord) <= effective_range {
                                attack_target.coord = Some(gp.coord);
                                next_phase.set(TurnPhase::ExecuteAction);
                                return;
                            }
                        }
                    }
                }
            }

            // 点击其他地方：待机
            next_phase.set(TurnPhase::WaitAction);
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
    for entity in &selected_query {
        commands.entity(entity).remove::<Selected>();
    }
    for marker in &range_markers {
        commands.entity(marker).despawn();
    }
    for h in &highlights {
        commands.entity(h).despawn();
    }

    // 标记所有玩家单位为已行动
    for mut unit in units.iter_mut() {
        if unit.faction == Faction::Player {
            unit.acted = true;
        }
    }

    next_phase.set(TurnPhase::TurnEnd);
}

/// 清除选中状态和范围标记
fn clear_selection(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    range_markers: &Query<(Entity, &GridPosition), Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for entity in selected_query {
        commands.entity(entity).remove::<Selected>();
    }
    for (marker, _) in range_markers {
        commands.entity(marker).despawn();
    }
    for h in highlights {
        commands.entity(h).despawn();
    }
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
    mut targets: Query<(Entity, &mut Unit, &GridPosition, &UnitName, &Transform), Without<Selected>>,
    tiles: Query<&Tile>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
    mut attack_target: ResMut<AttackTarget>,
    range_markers: Query<Entity, Or<(With<MovableRange>, With<AttackRange>)>>,
    highlights: Query<Entity, With<SelectionHighlight>>,
    mut combat_log: ResMut<CombatLog>,
    asset_server: Res<AssetServer>,
    map: Res<GameMap>,
) {
    // 清除范围标记和高亮
    for marker in &range_markers {
        commands.entity(marker).despawn();
    }
    for h in &highlights {
        commands.entity(h).despawn();
    }

    let font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    if let Ok((entity, mut unit, _pos, attacker_name)) = selected_units.single_mut() {
        // 查找攻击目标
        if let Some(target_coord) = attack_target.coord {
            for (target_entity, mut target, target_pos, target_name, target_transform) in targets.iter_mut() {
                if target_pos.coord == target_coord && target.faction != unit.faction {
                    let terrain = tiles
                        .iter()
                        .find_map(|t| {
                            if t.coord == target_pos.coord {
                                Some(t.terrain)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(Terrain::Plain);

                    let damage = calculate_damage(&unit, &target, terrain);
                    target.hp -= damage;

                    // 伤害数字弹出
                    let is_crit = unit.skill != Skill::None;
                    vfx::spawn_damage_popup(
                        &mut commands,
                        target_transform.translation.truncate(),
                        damage,
                        &font,
                        is_crit,
                    );

                    // 写入战斗日志（含技能名）
                    let attacker_color = if unit.faction == Faction::Player {
                        log_color::PLAYER
                    } else {
                        log_color::ENEMY
                    };
                    let defender_color = if target.faction == Faction::Player {
                        log_color::PLAYER
                    } else {
                        log_color::ENEMY
                    };
                    let skill_label = skill_name(&unit.skill);

                    let killed = target.hp <= 0;
                    combat_log.push(vec![
                        LogSegment { text: format!("[{}]", attacker_name.0), color: attacker_color },
                        LogSegment { text: format!(" 使用[{}]", skill_label), color: log_color::TURN },
                        LogSegment { text: " 攻击 ".to_string(), color: log_color::NORMAL },
                        LogSegment { text: format!("[{}]", target_name.0), color: defender_color },
                        LogSegment { text: " 造成 ".to_string(), color: log_color::NORMAL },
                        LogSegment { text: format!("[{}]", damage), color: log_color::DAMAGE },
                        LogSegment { text: " 伤害".to_string(), color: log_color::NORMAL },
                        LogSegment { text: format!(" ({})", terrain.label()), color: log_color::TERRAIN },
                    ]);

                    if killed {
                        combat_log.push(vec![
                            LogSegment { text: format!("[{}]", target_name.0), color: defender_color },
                            LogSegment { text: " 被击败！".to_string(), color: log_color::KILL },
                        ]);
                        commands.entity(target_entity).despawn();
                    }
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
    for marker in &range_markers {
        commands.entity(marker).despawn();
    }
    for h in &highlights {
        commands.entity(h).despawn();
    }

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
