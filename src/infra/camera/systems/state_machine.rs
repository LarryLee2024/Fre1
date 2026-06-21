//! State Machine — 镜头状态机 + CameraRequest 仲裁
//!
//! 包含三个核心系统：
//! 1. `process_camera_requests` — Observer 消费 CameraRequest，更新状态机和 TargetPose
//! 2. `idle_timeout` — FreeMove 空闲超时检测，超时后回 Idle
//! 3. `update_focus` — Focus 状态计时推进

use bevy::prelude::*;

use crate::infra::camera::components::{CameraInputBlock, CameraShake, IdleTimeout, MainCamera};
use crate::infra::camera::foundation::pose::{CameraPose, MAX_ZOOM, MIN_ZOOM, TargetPose};
use crate::infra::camera::foundation::request::CameraRequest;
use crate::infra::camera::foundation::state::CameraState;
use crate::infra::camera::foundation::target::CameraTarget;
use crate::infra::camera::resources::{TileSize, UnitPositionResolver};

use super::input_handler::has_camera_input;
use crate::infra::input::resources::InputState;

// ─── CameraTarget 解析 ───────────────────────────────────────

/// 将 CameraTarget 解析为世界坐标 Vec2。
fn resolve_target(target: &CameraTarget, resolver: &UnitPositionResolver, tile_size: f32) -> Vec2 {
    target.resolve(&|id| resolver.0.map(|f| f(id)), tile_size)
}

// ─── Observer: 处理外部 CameraRequest ─────────────────────────

/// 消费 CameraRequest 事件并更新状态机和 TargetPose。
///
/// 作为 Observer 注册在 CameraPlugin 中。外部系统通过
/// `commands.trigger(CameraRequest::...)` 触发此函数。
///
/// 状态转移规则（详见 docs/02-domain/infrastructure/camera_domain.md §2）：
/// - Focus 状态下静默忽略所有非 LockInput/UnlockInput 请求
/// - 无效转移静默忽略
pub fn process_camera_requests(
    trigger: On<CameraRequest>,
    camera_query: Query<(Entity, &CameraState, &TargetPose), With<MainCamera>>,
    input_block_query: Query<&CameraInputBlock, With<MainCamera>>,
    unit_resolver: Res<UnitPositionResolver>,
    tile_size: Res<TileSize>,
    mut commands: Commands,
) {
    let request = trigger.event();
    let Ok((entity, state, target)) = camera_query.single() else {
        return;
    };

    // Focus 状态下静默忽略（LockInput/UnlockInput 除外）
    if matches!(*state, CameraState::Focus { .. })
        && !matches!(
            request,
            CameraRequest::LockInput | CameraRequest::UnlockInput
        )
    {
        return;
    }

    match request {
        CameraRequest::MoveTo {
            target: t,
            duration: _,
        } => {
            let pos = resolve_target(t, &unit_resolver, tile_size.0);
            let mut new_pose = target.0.clone();
            new_pose.position = pos;
            commands.entity(entity).insert(TargetPose(new_pose));
        }
        CameraRequest::Follow { target: t } => {
            let pos = resolve_target(t, &unit_resolver, tile_size.0);
            let mut new_pose = target.0.clone();
            new_pose.position = pos;
            commands.entity(entity).insert(TargetPose(new_pose));
            commands
                .entity(entity)
                .insert(CameraState::Follow(t.clone()));
        }
        CameraRequest::Unfollow => {
            if matches!(*state, CameraState::Follow(_)) {
                commands.entity(entity).insert(CameraState::Idle);
            }
        }
        CameraRequest::SetZoom { zoom, duration: _ } => {
            let mut new_pose = target.0.clone();
            new_pose.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
            commands.entity(entity).insert(TargetPose(new_pose));
        }
        CameraRequest::Reset { duration: _ } => {
            commands
                .entity(entity)
                .insert(TargetPose(CameraPose::default()));
        }
        CameraRequest::Shake {
            intensity,
            duration,
        } => {
            if *duration <= 0.0 {
                return;
            }
            commands.entity(entity).insert(CameraShake::new(
                intensity.clamp(1.0, 20.0),
                duration.clamp(0.0, 5.0),
                0,
            ));
        }
        CameraRequest::LockInput => {
            let new_count = if let Ok(block) = input_block_query.single() {
                block.block_count.saturating_add(1)
            } else {
                1
            };
            commands.entity(entity).insert(CameraInputBlock {
                block_count: new_count,
            });
        }
        CameraRequest::UnlockInput => {
            if let Ok(block) = input_block_query.single() {
                let new_count = block.block_count.saturating_sub(1);
                if new_count == 0 {
                    commands.entity(entity).remove::<CameraInputBlock>();
                } else {
                    commands.entity(entity).insert(CameraInputBlock {
                        block_count: new_count,
                    });
                }
            }
        }
    }
}

// ─── IdleTimeout ──────────────────────────────────────────────

/// FreeMove 空闲超时检测——由 CameraPlugin 在 Update 中注册。
///
/// 仅在 CameraState == FreeMove 时运行。检查当前帧是否有 Camera 输入：
/// - 有输入 → 重置 elapsed 为 0
/// - 无输入 → elapsed += delta_seconds
/// - elapsed >= timeout_duration → CameraState 回到 Idle
pub fn idle_timeout(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut camera_query: Query<(Entity, &mut CameraState, &mut IdleTimeout), With<MainCamera>>,
    mut commands: Commands,
) {
    for (entity, mut state, mut timeout) in camera_query.iter_mut() {
        if !matches!(*state, CameraState::FreeMove) {
            continue;
        }

        // 检查当前帧是否有 Camera 输入
        if has_camera_input(&input_state) {
            timeout.elapsed = 0.0;
        } else {
            timeout.elapsed += time.delta().as_secs_f32();
        }

        // 超时 → 回到 Idle
        if timeout.elapsed >= timeout.timeout_duration {
            *state = CameraState::Idle;
            commands.entity(entity).remove::<IdleTimeout>();
        }
    }
}

// ─── Focus 计时 ───────────────────────────────────────────────

/// 推进 Focus 状态计时——由 CameraPlugin 在 Update 中注册。
///
/// 每帧更新 Focus.elapsed，达到 duration 后自动回到 Idle。
pub fn update_focus(
    time: Res<Time>,
    mut camera_query: Query<(&mut CameraState, &mut TargetPose), With<MainCamera>>,
    unit_resolver: Res<UnitPositionResolver>,
    tile_size: Res<TileSize>,
) {
    let Ok((mut state, mut target)) = camera_query.single_mut() else {
        return;
    };

    // 提取 Focus 字段值（先复制以释放 state 的借用）
    let (focus_target, duration, elapsed) = match state.as_ref() {
        CameraState::Focus {
            target,
            duration,
            elapsed,
        } => (*target, *duration, *elapsed),
        _ => return,
    };

    // 推进计时
    let new_elapsed = elapsed + time.delta().as_secs_f32();
    let done = new_elapsed >= duration;

    // 每帧更新 TargetPose（目标可能在移动）
    let pos = resolve_target(&focus_target, &unit_resolver, tile_size.0);
    target.0.position = pos;

    if done {
        // Focus 完成 → 回到 Idle，保持当前位置
        *state = CameraState::Idle;
    } else {
        // 更新计时
        *state = CameraState::Focus {
            target: focus_target,
            duration,
            elapsed: new_elapsed,
        };
    }
}
