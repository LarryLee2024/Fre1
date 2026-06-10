// Gizmos 调试可视化系统：寻路路径、AI 决策、占用网格、范围轮廓
// 遵循铁律：复杂系统必须有可视化观察窗口
// Gizmos rect_2d 只画线框轮廓，不画填充矩形，与 Sprite 高亮互补

use crate::battle::CombatIntent;
use crate::character::{AttackRange, Faction, GridPosition, MovableRange, MovingUnit, Unit};
use crate::map::GameMap;
use crate::map::runtime::OccupancyGrid;
use crate::ui::theme::UiTheme;

use super::overlay::DebugOverlay;
use bevy::prelude::*;

// ── 寻路路径调试 ──

/// 绘制移动单位的路径：起点→终点的线框矩形序列
pub fn debug_pathfinding(
    mut gizmos: Gizmos,
    overlay: Res<DebugOverlay>,
    map: Res<GameMap>,
    moving_units: Query<&MovingUnit>,
) {
    if !overlay.show_pathfinding {
        return;
    }

    let tile_size = map.tile_size;
    let size = Vec2::splat(tile_size - 4.0);
    let color = Color::srgb(0.0, 1.0, 0.5);

    for moving in &moving_units {
        for (i, &coord) in moving.path.iter().enumerate() {
            let world_pos = map.coord_to_world(coord);
            let iso = Isometry2d::from_translation(world_pos);

            // 路径起点用不同颜色区分
            let c = if i == 0 {
                Color::srgb(0.2, 1.0, 0.2)
            } else if i == moving.path.len() - 1 {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                color
            };

            gizmos.rect_2d(iso, size, c);
        }
    }
}

// ── AI 决策调试 ──

/// 绘制 AI 战斗意图：攻击目标位置、攻击者位置
pub fn debug_ai_intent(
    mut gizmos: Gizmos,
    overlay: Res<DebugOverlay>,
    map: Res<GameMap>,
    combat_intent: Res<CombatIntent>,
    units: Query<(Entity, &Unit, &GridPosition), Without<crate::character::Dead>>,
) {
    if !overlay.show_ai_intent {
        return;
    }

    let tile_size = map.tile_size;
    let size = Vec2::splat(tile_size - 2.0);

    // 绘制攻击者位置（蓝色菱形轮廓 → 用 rect_2d 替代）
    if let Some(source_entity) = combat_intent.source_entity {
        if let Ok((_, unit, gp)) = units.get(source_entity) {
            if unit.faction == Faction::Enemy {
                let world_pos = map.coord_to_world(gp.coord);
                let iso = Isometry2d::from_translation(world_pos);
                gizmos.rect_2d(iso, size, Color::srgb(0.3, 0.6, 1.0));
            }
        }
    }

    // 绘制攻击目标位置（红色轮廓）
    if let Some(target_coord) = combat_intent.target_coord {
        let world_pos = map.coord_to_world(target_coord);
        let iso = Isometry2d::from_translation(world_pos);
        // 画双层轮廓增强可见性
        gizmos.rect_2d(iso, size, Color::srgb(1.0, 0.2, 0.1));
        gizmos.rect_2d(
            iso,
            Vec2::splat(tile_size + 4.0),
            Color::srgb(1.0, 0.5, 0.3),
        );
    }
}

// ── 占用网格调试 ──

/// 绘制被占据的格子：小方块标记
pub fn debug_occupancy(
    mut gizmos: Gizmos,
    overlay: Res<DebugOverlay>,
    map: Res<GameMap>,
    occupancy: Res<OccupancyGrid>,
    units: Query<(Entity, &Unit, &GridPosition)>,
) {
    if !overlay.show_occupancy {
        return;
    }

    let tile_size = map.tile_size;
    // 小标记尺寸，不遮挡格子内容
    let marker_size = Vec2::splat(tile_size * 0.25);

    // 构建 Entity → Faction 映射，用于颜色区分
    let faction_map: std::collections::HashMap<Entity, Faction> =
        units.iter().map(|(e, u, _)| (e, u.faction)).collect();

    for coord in occupancy.occupied_coords() {
        let world_pos = map.coord_to_world(coord);
        // 偏移到格子右上角，避免遮挡格子中心
        let offset_pos = world_pos + Vec2::new(tile_size * 0.3, tile_size * 0.3);
        let iso = Isometry2d::from_translation(offset_pos);

        // 根据阵营选择颜色
        let color = occupancy
            .get_entity(coord)
            .and_then(|e| faction_map.get(&e))
            .map(|&f| match f {
                Faction::Player => Color::srgb(0.3, 0.6, 1.0),
                Faction::Enemy => Color::srgb(1.0, 0.3, 0.2),
            })
            .unwrap_or(Color::srgb(0.7, 0.7, 0.7));

        gizmos.rect_2d(iso, marker_size, color);
    }
}

// ── 攻击/移动范围轮廓线调试 ──

/// 绘制攻击/移动范围的线框轮廓，补充 Sprite 填充高亮
/// Sprite 高亮负责半透明填充，Gizmos 轮廓负责清晰边界
pub fn debug_range_outline(
    mut gizmos: Gizmos,
    overlay: Res<DebugOverlay>,
    map: Res<GameMap>,
    theme: Res<UiTheme>,
    move_range: Query<&GridPosition, With<MovableRange>>,
    attack_range: Query<&GridPosition, With<AttackRange>>,
) {
    if !overlay.show_range_outline {
        return;
    }

    let tile_size = map.tile_size;
    let size = Vec2::splat(tile_size);

    // 移动范围轮廓（蓝色线框）
    let move_color = Color::srgb(0.3, 0.6, 1.0);
    for gp in &move_range {
        let world_pos = map.coord_to_world(gp.coord);
        let iso = Isometry2d::from_translation(world_pos);
        gizmos.rect_2d(iso, size, move_color);
    }

    // 攻击范围轮廓（红色线框）
    let attack_color = Color::srgb(1.0, 0.3, 0.2);
    for gp in &attack_range {
        let world_pos = map.coord_to_world(gp.coord);
        let iso = Isometry2d::from_translation(world_pos);
        gizmos.rect_2d(iso, size, attack_color);
    }
}
