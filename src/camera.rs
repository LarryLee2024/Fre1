// 相机控制模块：WASD/方向键平移，滚轮缩放

use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// 相机移动速度（像素/秒）
const CAMERA_MOVE_SPEED: f32 = 300.0;
/// 缩放速度
const CAMERA_ZOOM_SPEED: f32 = 0.1;
/// 最小缩放
const CAMERA_ZOOM_MIN: f32 = 0.3;
/// 最大缩放
const CAMERA_ZOOM_MAX: f32 = 3.0;

/// 相机控制标记
#[derive(Component)]
pub struct CameraController;

/// 相机管理插件
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::{AppState, GameSet};
        app.add_systems(
            OnEnter(AppState::InGame),
            spawn_camera.in_set(GameSet::Camera),
        )
        .add_systems(Update, camera_control);
    }
}

/// 生成带控制器的相机
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, CameraController));
}

/// 相机平移和缩放
pub fn camera_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<CameraController>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    // WASD / 方向键平移
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        // 根据缩放级别调整移动速度
        let scale = transform.scale.x;
        transform.translation.x += direction.x * CAMERA_MOVE_SPEED * time.delta_secs() / scale;
        transform.translation.y += direction.y * CAMERA_MOVE_SPEED * time.delta_secs() / scale;
    }

    // 滚轮缩放
    for event in scroll_events.read() {
        let zoom_delta = -event.y * CAMERA_ZOOM_SPEED;
        let new_scale = (transform.scale.x + zoom_delta).clamp(CAMERA_ZOOM_MIN, CAMERA_ZOOM_MAX);
        transform.scale = Vec3::splat(new_scale);
    }
}
