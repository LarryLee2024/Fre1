// 移动动画系统：导航路径线 + 箭头 + 平滑移动

use crate::core::map::GameMap;
use crate::core::turn::TurnPhase;
use bevy::picking::prelude::Pickable;
use bevy::prelude::*;

use super::{GridPosition, MovingUnit, PathArrow};

/// 路径线颜色
const LINE_COLOR: Color = Color::srgba(1.0, 1.0, 0.4, 0.7);
/// 箭头颜色
const ARROW_HEAD_COLOR: Color = Color::srgba(1.0, 1.0, 0.4, 0.85);
/// 线条宽度
const LINE_WIDTH: f32 = 4.0;
/// 箭头大小
const ARROW_HEAD_SIZE: f32 = 14.0;

/// 生成导航路径（连续线段 + 末端箭头）
pub fn spawn_path_arrows(commands: &mut Commands, map: &GameMap, path: &[IVec2]) {
    if path.is_empty() {
        return;
    }

    // 将路径坐标转为世界坐标
    let world_points: Vec<Vec3> = path
        .iter()
        .map(|&coord| {
            let w = map.coord_to_world(coord);
            Vec3::new(w.x, w.y, 0.5)
        })
        .collect();

    // 为每对相邻路径点生成线段
    for i in 0..world_points.len().saturating_sub(1) {
        spawn_line_segment(commands, world_points[i], world_points[i + 1]);
    }

    // 末端箭头
    if world_points.len() >= 2 {
        let last = world_points[world_points.len() - 1];
        let prev = world_points[world_points.len() - 2];
        spawn_arrow_head(commands, prev, last);
    }
}

/// 生成两点之间的线段 sprite
fn spawn_line_segment(commands: &mut Commands, start: Vec3, end: Vec3) {
    let mid = (start + end) / 2.0;
    let diff = end - start;
    let length = diff.length();
    let angle = diff.y.atan2(diff.x);

    commands.spawn((
        Sprite {
            color: LINE_COLOR,
            custom_size: Some(Vec2::new(length, LINE_WIDTH)),
            ..default()
        },
        Transform::from_translation(mid).with_rotation(Quat::from_rotation_z(angle)),
        PathArrow,
        Pickable::IGNORE,
    ));
}

/// 生成末端箭头（菱形压扁成箭头形状）
fn spawn_arrow_head(commands: &mut Commands, from: Vec3, to: Vec3) {
    let diff = to - from;
    let angle = diff.y.atan2(diff.x);

    commands.spawn((
        Sprite {
            color: ARROW_HEAD_COLOR,
            custom_size: Some(Vec2::new(ARROW_HEAD_SIZE, ARROW_HEAD_SIZE)),
            ..default()
        },
        Transform::from_translation(to)
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(1.0, 0.5, 1.0)), // 压扁成箭头形状
        PathArrow,
        Pickable::IGNORE,
    ));
}

/// 清除所有导航箭头
pub fn despawn_path_arrows(commands: &mut Commands, arrows: Query<Entity, With<PathArrow>>) {
    for entity in &arrows {
        commands.entity(entity).try_despawn();
    }
}

/// 移动动画系统：逐格插值移动
/// 每帧更新 MovingUnit 的 Transform，到达一格后前进到下一格
/// 全部走完后移除 MovingUnit 组件，更新 GridPosition，切换阶段
pub fn animate_movement(
    mut commands: Commands,
    time: Res<Time>,
    map: Res<GameMap>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut moving_units: Query<(Entity, &mut MovingUnit, &mut Transform, &mut GridPosition)>,
    arrows: Query<Entity, With<PathArrow>>,
) {
    for (entity, mut moving, mut transform, mut gp) in &mut moving_units {
        if moving.is_finished() {
            continue;
        }

        // 获取当前目标格的世界坐标
        let Some(target_coord) = moving.target_coord() else {
            continue;
        };
        let target_world = map.coord_to_world(target_coord);

        // 累加时间
        moving.elapsed += time.delta_secs();

        // 计算插值进度
        let t = (moving.elapsed / moving.speed).min(1.0);

        // 获取起点世界坐标（上一格的位置）
        let start_coord = if moving.current_index > 0 {
            moving.path[moving.current_index - 1]
        } else {
            gp.coord
        };
        let start_world = map.coord_to_world(start_coord);

        // 线性插值
        transform.translation.x = start_world.x + (target_world.x - start_world.x) * t;
        transform.translation.y = start_world.y + (target_world.y - start_world.y) * t;

        // 到达当前目标格
        if t >= 1.0 {
            gp.coord = target_coord;
            transform.translation.x = target_world.x;
            transform.translation.y = target_world.y;
            moving.current_index += 1;
            moving.elapsed = 0.0;
        }

        // 全部走完
        if moving.is_finished() {
            // 确保最终位置精确
            let final_coord = moving.path.last().copied().unwrap_or(gp.coord);
            let final_world = map.coord_to_world(final_coord);
            transform.translation.x = final_world.x;
            transform.translation.y = final_world.y;
            gp.coord = final_coord;

            // 切换到回调阶段
            let phase = moving.next_phase;
            commands.entity(entity).remove::<MovingUnit>();
            next_phase.set(phase);

            // 清除导航箭头
            for arrow in &arrows {
                commands.entity(arrow).try_despawn();
            }
        }
    }
}
