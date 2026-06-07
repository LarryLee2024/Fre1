// 输入处理模块：点击选择、移动、攻击

use bevy::prelude::*;
use crate::map::{GameMap, Tile, Terrain};
use crate::unit::{Unit, Faction, GridPosition, Selected, MovableRange, AttackRange};
use crate::turn::{TurnState, TurnPhase};
use crate::pathfinding::{find_reachable_tiles, build_tile_terrain_map};
use crate::combat::{manhattan_distance, calculate_damage};

/// 点击选择单位或移动
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
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) else { return };

    let coord = map.world_to_coord(world_pos);
    if !map.is_in_bounds(coord) {
        return;
    }

    match turn_phase.get() {
        TurnPhase::SelectUnit => {
            // 只能选择当前阵营且未行动的单位
            for (entity, unit, gp, _) in &units {
                if unit.faction == turn_state.current_faction
                    && !unit.acted
                    && gp.coord == coord
                {
                    // 取消之前的选中
                    for prev in &selected_query {
                        commands.entity(prev).remove::<Selected>();
                    }
                    // 清除范围标记
                    for (marker, _) in &range_markers {
                        commands.entity(marker).despawn();
                    }

                    commands.entity(entity).insert(Selected);
                    next_phase.set(TurnPhase::MoveUnit);

                    // 显示可移动范围
                    show_move_range(&mut commands, &map, &tiles, &units, unit, gp.coord);
                    return;
                }
            }
        }
        TurnPhase::MoveUnit => {
            // 检查点击的格子是否在可移动范围内
            let is_movable = range_markers
                .iter()
                .any(|(_, gp)| gp.coord == coord);

            if is_movable {
                // 移动选中的单位到目标格子
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, _, _, _)) = units.get(selected_entity) {
                        let world_pos = map.coord_to_world(coord);
                        commands.entity(selected_entity)
                            .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                            .insert(GridPosition { coord });
                    }
                }
            }

            // 清除移动范围标记，进入选择行动
            for (marker, _) in &range_markers {
                commands.entity(marker).despawn();
            }
            next_phase.set(TurnPhase::SelectAction);
        }
        TurnPhase::SelectAction => {
            // 简化处理：点击任意位置执行攻击或结束行动
            for (marker, _) in &range_markers {
                commands.entity(marker).despawn();
            }
            next_phase.set(TurnPhase::ExecuteAction);
        }
        _ => {}
    }
}

/// 显示可移动范围
fn show_move_range(
    commands: &mut Commands,
    map: &GameMap,
    tiles: &Query<&Tile>,
    units: &Query<(Entity, &Unit, &GridPosition, &Transform)>,
    unit: &Unit,
    start_coord: IVec2,
) {
    let terrain_map = build_tile_terrain_map(tiles);

    // 构建占位表需要单独查询
    let occupation_units: Vec<(IVec2, Faction)> = units
        .iter()
        .map(|(_, u, gp, _)| (gp.coord, u.faction))
        .collect();
    let occupation_map: std::collections::HashMap<IVec2, bool> = occupation_units
        .iter()
        .filter(|(_, f)| *f != unit.faction)
        .map(|(coord, _)| (*coord, true))
        .collect();

    let reachable = find_reachable_tiles(
        start_coord,
        unit.mov,
        map,
        &terrain_map,
        &occupation_map,
    );

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

/// 执行行动：攻击或待机
pub fn execute_action(
    mut selected_units: Query<(Entity, &mut Unit, &GridPosition), With<Selected>>,
    mut targets: Query<(Entity, &mut Unit, &GridPosition), Without<Selected>>,
    tiles: Query<&Tile>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut commands: Commands,
) {
    if let Ok((entity, mut unit, pos)) = selected_units.single_mut() {
        // 尝试攻击范围内的敌人
        for (target_entity, mut target, target_pos) in targets.iter_mut() {
            if target.faction != unit.faction
                && manhattan_distance(pos.coord, target_pos.coord) <= unit.attack_range
            {
                // 获取防御方地形
                let terrain = tiles.iter().find_map(|t| {
                    if t.coord == target_pos.coord { Some(t.terrain) } else { None }
                }).unwrap_or(Terrain::Plain);

                let damage = calculate_damage(&unit, &target, terrain);
                target.hp -= damage;

                if target.hp <= 0 {
                    commands.entity(target_entity).despawn();
                }
                break;
            }
        }

        // 标记已行动
        unit.acted = true;
        commands.entity(entity).remove::<Selected>();
    }

    next_phase.set(TurnPhase::TurnEnd);
}

/// 回合结束处理
pub fn handle_turn_end(
    mut turn_state: ResMut<TurnState>,
    mut units: Query<&mut Unit>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
) {
    // 检查当前阵营是否所有单位都已行动
    let all_acted = units
        .iter()
        .filter(|u| u.faction == turn_state.current_faction)
        .all(|u| u.acted);

    if all_acted {
        // 切换阵营
        let next_faction = match turn_state.current_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => {
                // 敌方回合结束，新回合开始
                turn_state.turn_number += 1;
                Faction::Player
            }
        };
        turn_state.current_faction = next_faction;

        // 重置新阵营单位的行动状态
        for mut unit in &mut units {
            if unit.faction == next_faction {
                unit.acted = false;
            }
        }
    }

    next_phase.set(TurnPhase::SelectUnit);
}
