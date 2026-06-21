//! Input Handler — 消费 InputAction，转换为内部镜头移动
//!
//! 在 PreUpdate 中调度。读取 InputState Resource 中的 Camera 相关 InputAction，
//! 直接更新 TargetPose 和 CameraState（不经过 CameraRequest Event 管道）。
//!
//! 输入处理规则：
//!   - Idle 时任何 Camera 输入 → FreeMove 状态
//!   - FreeMove 时输入增量移动（IdleTimeout 由 idle_timeout system 管理）
//!   - Follow 时用户输入 → FreeMove 状态（用户覆盖）
//!   - Focus 时忽略所有 FreeMove 输入（输入锁定）
//!   - CameraInputBlock 存在且 block_count > 0 时忽略所有 FreeMove 输入

use bevy::prelude::*;

use crate::infra::camera::components::CameraInputBlock;
use crate::infra::camera::components::IdleTimeout;
use crate::infra::camera::components::MainCamera;
use crate::infra::camera::foundation::pose::{
    CAMERA_MOVE_SPEED, MAX_ZOOM, MIN_ZOOM, TargetPose, ZOOM_STEP_FACTOR,
};
use crate::infra::camera::foundation::state::CameraState;
use crate::infra::input::action::InputAction;
use crate::infra::input::resources::InputState;

/// 处理自由移动输入——由 CameraPlugin 在 PreUpdate 中注册。
pub fn handle_camera_input(
    input_state: Res<InputState>,
    time: Res<Time>,
    mut camera_query: Query<
        (
            Entity,
            &mut CameraState,
            &mut TargetPose,
            Option<&CameraInputBlock>,
        ),
        With<MainCamera>,
    >,
    mut commands: Commands,
) {
    let Ok((entity, mut state, mut target, input_block)) = camera_query.single_mut() else {
        return;
    };

    // ── 检查输入锁定 ──
    if matches!(*state, CameraState::Focus { .. }) {
        return; // Focus 状态下输入锁定
    }

    // 检查 CameraInputBlock Component
    if let Some(block) = input_block {
        if block.block_count > 0 {
            return;
        }
    }

    // ── 采集方向输入 ──
    let mut direction = Vec2::ZERO;
    if input_state.pressed(InputAction::CameraUp) {
        direction.y += 1.0;
    }
    if input_state.pressed(InputAction::CameraDown) {
        direction.y -= 1.0;
    }
    if input_state.pressed(InputAction::CameraLeft) {
        direction.x -= 1.0;
    }
    if input_state.pressed(InputAction::CameraRight) {
        direction.x += 1.0;
    }

    let has_movement = direction != Vec2::ZERO;

    // ── 缩放输入 ──
    let has_zoom_input = if input_state.just_pressed(InputAction::CameraZoomIn) {
        target.0.zoom = (target.0.zoom * ZOOM_STEP_FACTOR).min(MAX_ZOOM);
        true
    } else if input_state.just_pressed(InputAction::CameraZoomOut) {
        target.0.zoom = (target.0.zoom / ZOOM_STEP_FACTOR).max(MIN_ZOOM);
        true
    } else {
        false
    };

    // 没有移动或缩放输入 → 保持当前状态（idle_timeout system 处理超时）
    if !has_movement && !has_zoom_input {
        return;
    }

    // ── 有方向输入：更新 TargetPose ──
    if has_movement {
        let delta = direction.normalize_or_zero() * CAMERA_MOVE_SPEED * time.delta().as_secs_f32();
        target.0.position += delta;
    }

    // ── 状态转移 ──
    match *state {
        CameraState::Idle => {
            *state = CameraState::FreeMove;
            commands.entity(entity).insert(IdleTimeout::default());
        }
        CameraState::FreeMove => {
            // IdleTimeout 会在 idle_timeout system 中被自动重置
            //（因为 idle_timeout 每帧检查 InputState 是否有输入活动）
        }
        CameraState::Follow(_) => {
            *state = CameraState::FreeMove;
            commands.entity(entity).insert(IdleTimeout::default());
        }
        CameraState::Focus { .. } => {
            // 不应到达这里（已提前 return）
        }
    }
}

/// 检查当前帧是否有摄像机输入活动（纯函数，供 idle_timeout 等系统使用）。
pub fn has_camera_input(input_state: &InputState) -> bool {
    input_state.pressed(InputAction::CameraUp)
        || input_state.pressed(InputAction::CameraDown)
        || input_state.pressed(InputAction::CameraLeft)
        || input_state.pressed(InputAction::CameraRight)
        || input_state.just_pressed(InputAction::CameraZoomIn)
        || input_state.just_pressed(InputAction::CameraZoomOut)
}
