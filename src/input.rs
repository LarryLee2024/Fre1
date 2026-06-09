// 输入处理模块：点击选择、移动、攻击分发
// 通过 UiCommand Message 发出用户意图，不直接修改游戏状态

use crate::character::{
    AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit,
};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::tag::GameplayTags;
use crate::map::GameMap;
use crate::skill::SkillSlots;
use crate::turn::{TurnPhase, TurnState};
use crate::ui::events::UiCommand;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// 处理玩家点击：发送 UiCommand Message
pub fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    units: Query<(Entity, &Unit, &GridPosition)>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }

    let left_clicked = mouse_button.just_pressed(MouseButton::Left);
    let right_clicked = mouse_button.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    // 右键在 MoveUnit 阶段发送取消
    if right_clicked {
        if *turn_phase.get() == TurnPhase::MoveUnit {
            ui_commands.write(UiCommand::Cancel);
            return;
        }
        if *turn_phase.get() == TurnPhase::ActionMenu
            || *turn_phase.get() == TurnPhase::SelectTarget
        {
            return;
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
            for (entity, unit, gp) in &units {
                if unit.faction == Faction::Player && !unit.acted && gp.coord == coord {
                    ui_commands.write(UiCommand::SelectUnit { entity });
                    return;
                }
            }
        }
        TurnPhase::MoveUnit => {
            ui_commands.write(UiCommand::MoveUnit { coord });
        }
        TurnPhase::SelectTarget => {
            ui_commands.write(UiCommand::SelectTarget { coord });
        }
        _ => {}
    }
}

/// 处理右键取消（ActionMenu/SelectTarget 阶段）
pub fn handle_right_cancel(
    mouse_button: Res<ButtonInput<MouseButton>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    match turn_phase.get() {
        TurnPhase::ActionMenu | TurnPhase::SelectTarget => {
            ui_commands.write(UiCommand::Cancel);
        }
        _ => {}
    }
}

/// 按 E 键结束回合
pub fn handle_end_turn(
    keyboard: Res<ButtonInput<KeyCode>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
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

    ui_commands.write(UiCommand::EndTurn);
}

// ── 公共辅助函数（供 command_handler 调用）──

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

/// 清除范围标记和高亮
pub fn clear_markers(
    commands: &mut Commands,
    range_entities: &Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    crate::character::clear_markers(commands, range_entities, highlights);
}

/// 清除选中状态和范围标记
pub fn clear_selection(
    commands: &mut Commands,
    selected_query: &Query<Entity, With<Selected>>,
    range_entities: &Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
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
    terrain_map: &std::collections::HashMap<IVec2, (crate::map::Terrain, Option<u32>)>,
    units: &Query<(
        Entity,
        &Unit,
        &GridPosition,
        &Transform,
        &Attributes,
        &SkillSlots,
        &GameplayTags,
    )>,
    unit: &Unit,
    start_coord: IVec2,
    calculator: &dyn crate::map::TerrainCostCalculator,
) {
    use crate::map::find_reachable_tiles;
    use crate::ui::theme::UiTheme;

    let theme = UiTheme::default();

    let occupation_units: Vec<(IVec2, Faction)> = units
        .iter()
        .map(|(_, u, gp, _, _, _, _)| (gp.coord, u.faction))
        .collect();
    let occupation_map: std::collections::HashMap<IVec2, bool> = occupation_units
        .iter()
        .filter(|(_, f)| *f != unit.faction)
        .map(|(coord, _)| (*coord, true))
        .collect();

    let move_points = units
        .iter()
        .find(|(_, u, gp, _, _, _, _)| u.faction == unit.faction && gp.coord == start_coord)
        .map(|(_, _, _, _, attrs, _, _)| attrs.get(AttributeKind::MoveRange) as u32)
        .unwrap_or(3);

    let reachable = find_reachable_tiles(
        start_coord,
        move_points,
        map,
        terrain_map,
        &occupation_map,
        calculator,
    );
    let tile_size = map.tile_size;

    for (coord, _) in reachable {
        let world_pos = map.coord_to_world(coord);
        commands.spawn((
            Sprite {
                color: theme.movable_range,
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
    use crate::ui::theme::UiTheme;

    let theme = UiTheme::default();
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
                    color: theme.attack_range,
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
    use crate::ui::theme::UiTheme;

    let theme = UiTheme::default();
    let world_pos = map.coord_to_world(coord);
    let tile_size = map.tile_size;
    commands.spawn((
        Sprite {
            color: theme.selection_highlight,
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
        app.add_systems(Update, handle_click.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                handle_right_cancel.run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, handle_end_turn.run_if(in_state(AppState::InGame)));
    }
}
