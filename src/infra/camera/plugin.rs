//! CameraPlugin — Camera 系统 ECS Plugin
//!
//! 按 ADR-064 §8 规定的 Schedule 顺序注册系统：
//! - PreUpdate:  input_handler::handle_camera_input
//! - Update:     state_machine::process_camera_requests (Observer)
//! - Update:     state_machine::idle_timeout
//! - Update:     state_machine::update_focus
//! - PostUpdate: movement::interpolate_pose
//! - PostUpdate: bounds::clamp_position
//! - PostUpdate: shake::apply_shake
//! - PostUpdate: movement::write_to_transform

use bevy::prelude::*;

use super::components::{CameraInputBlock, CameraShake, IdleTimeout, MainCamera};
use super::foundation::pose::{CameraPose, CurrentPose, TargetPose, Z_CAMERA};
use super::foundation::request::CameraRequest;
use super::foundation::state::CameraState;
use super::resources::{TileSize, UnitPositionResolver};
use super::systems::{bounds, input_handler, movement, shake, state_machine};

/// CameraPlugin — 镜头基础设施 Plugin。
///
/// 在 Phase 8（Input 之后）注册到 App。
/// 不负责 Camera Entity 的 spawn——由场景系统在 OnEnter 时调用 `spawn_camera` 工厂函数。
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // ── 1. 注册类型 ──
        app.register_type::<CameraPose>();
        app.register_type::<TargetPose>();
        app.register_type::<CurrentPose>();
        app.register_type::<CameraState>();
        app.register_type::<CameraBounds>();
        app.register_type::<CameraShake>();
        app.register_type::<CameraInputBlock>();
        app.register_type::<IdleTimeout>();
        app.register_type::<MainCamera>();

        // ── 2. 注册 Resources ──
        app.init_resource::<UnitPositionResolver>();
        app.init_resource::<TileSize>();

        // ── 3. 注册 Observer（消费 CameraRequest） ──
        app.add_observer(state_machine::process_camera_requests);

        // ── 4. 注册 System（按调度顺序） ──
        // PreUpdate: 输入处理（在 InputState 可用之后）
        app.add_systems(PreUpdate, input_handler::handle_camera_input);

        // Update: 状态机 + 超时检测 + Focus 计时
        app.add_systems(
            Update,
            (state_machine::idle_timeout, state_machine::update_focus),
        );

        // PostUpdate: Pose 插值 → 边界钳位 → 震屏 → Transform 写入
        app.add_systems(
            PostUpdate,
            (
                movement::interpolate_pose,
                bounds::clamp_position,
                shake::apply_shake,
                movement::write_to_transform,
            ),
        );

        tracing::info!(target: "camera", "[CameraPlugin] 已初始化");
    }
}

// ─── Camera Entity 工厂函数 ─────────────────────────────────

/// CameraBounds 组件——镜头边界约束（挂在 Camera Entity 上）。
///
/// 由场景系统在 OnEnter 时插入。详见 `camera_domain.md` §11。
pub use super::components::CameraBounds;

/// 生成 Camera Entity 的工厂函数——供场景 OnEnter System 调用。
///
/// 创建 Camera Entity 并附加：
/// - Camera2d / Camera / MainCamera 标记
/// - TargetPose / CurrentPose（初始值为 default_pose）
/// - CameraState::Idle
/// - Transform / GlobalTransform
///
/// 场景系统后续可可选插入 CameraBounds。
///
/// # 示例
/// ```ignore
/// // 在场景 OnEnter System 中：
/// fn spawn_camera_for_scene(mut commands: Commands) {
///     let camera_id = CameraPlugin::spawn_camera(
///         &mut commands,
///         CameraPose {
///             position: Vec2::new(400.0, 300.0),
///             zoom: 1.0,
///             rotation: 0.0,
///         },
///     );
///     // 可选：设置边界
///     commands.entity(camera_id).insert(CameraBounds {
///         min: Vec2::ZERO,
///         max: Vec2::new(800.0, 600.0),
///     });
/// }
/// ```
pub fn spawn_camera(commands: &mut Commands, default_pose: CameraPose) -> Entity {
    commands
        .spawn((
            Camera2d,
            MainCamera,
            TargetPose(default_pose.clone()),
            CurrentPose(default_pose),
            CameraState::Idle,
            Transform::from_xyz(0.0, 0.0, Z_CAMERA),
        ))
        .id()
}

/// 销毁 Camera Entity——供场景 OnExit System 调用。
///
/// 如果 Camera Entity 已不存在，则不执行任何操作（幂等性保证）。
pub fn despawn_camera(mut commands: Commands, camera_query: Query<Entity, With<MainCamera>>) {
    if let Ok(entity) = camera_query.single() {
        commands.entity(entity).despawn();
        tracing::debug!(target: "camera", "[despawn_camera] Camera Entity 已销毁");
    }
}
