// 高亮与标记表现层：范围显示、选中高亮、标记清除
// 逻辑层通过 Message 通知，表现层自行决定如何高亮
// 修复：所有函数接收 &UiTheme 参数，响应运行时主题变更

use crate::character::{
    AttackRange, GridPosition, MovableRange, Selected, SelectionHighlight, Unit,
};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::GameplayTags;
use crate::map::GameMap;
use crate::map::TerrainRegistry;
use crate::map::runtime::{OccupancyGrid, TerrainGrid};
use crate::skill::SkillSlots;
use crate::ui::theme::UiTheme;
use bevy::prelude::*;

/// 重新导出 clear_markers，方便调用方统一从 highlight 模块导入
pub use crate::character::clear_markers;

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
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    occupancy: &OccupancyGrid,
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
    theme: &UiTheme,
) {
    use crate::map::find_reachable_tiles;

    let move_points = units
        .iter()
        .find(|(_, u, gp, _, _, _, _)| u.faction == unit.faction && gp.coord == start_coord)
        .map(|(_, _, _, _, attrs, _, _)| attrs.get(AttributeKind::MoveRange) as u32)
        .unwrap_or(3);

    let reachable = find_reachable_tiles(
        start_coord,
        move_points,
        map,
        terrain_grid,
        terrain_registry,
        occupancy,
        None,
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
pub fn show_attack_range(
    commands: &mut Commands,
    map: &GameMap,
    center: IVec2,
    range: u32,
    theme: &UiTheme,
) {
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
pub fn spawn_selection_highlight(
    commands: &mut Commands,
    map: &GameMap,
    coord: IVec2,
    theme: &UiTheme,
) {
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
