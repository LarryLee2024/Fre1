//! Movement — Pose 插值管线 + Transform 写入
//!
//! 在 PostUpdate 中调度。处理：
//! 1. CurrentPose 向 TargetPose 的帧率无关线性插值
//! 2. 最终 CurrentPose 写入 Bevy Transform 和 OrthographicProjection

use bevy::prelude::*;

use crate::infra::camera::components::MainCamera;
use crate::infra::camera::foundation::pose::{CurrentPose, LERP_SPEED, TargetPose, Z_CAMERA};

/// 插值 CurrentPose 向 TargetPose 逼近——由 CameraPlugin 在 PostUpdate 中注册。
///
/// 使用帧率无关的线性插值：CurrentPose = CurrentPose.lerp(TargetPose, t)
/// 其中 t = LERP_SPEED * delta_seconds，上限为 1.0。
/// 插值是确定性数学运算，不依赖随机数。
pub fn interpolate_pose(
    time: Res<Time>,
    mut camera_query: Query<(&TargetPose, &mut CurrentPose), With<MainCamera>>,
) {
    let Ok((target, mut current)) = camera_query.single_mut() else {
        return;
    };

    let t = (LERP_SPEED * time.delta().as_secs_f32()).min(1.0);
    current.0 = current.0.lerp(&target.0, t);
}

/// 将 CurrentPose 写入 Bevy Transform。
///
/// 在 PostUpdate 最后一步执行。写入内容：
/// - Transform.translation.xy = CurrentPose.position（z 固定为场景深度常数）
///
/// 如果 Camera Entity 缺少必要的 Transform 组件，跳过写入并记录警告。
/// TODO[P2][Camera][2026-07-01]: 写入 zoom 到 OrthographicProjection.scale（Bevy 0.19 API）
pub fn write_to_transform(
    mut camera_query: Query<(&CurrentPose, &mut Transform), With<MainCamera>>,
) {
    let Ok((pose, mut transform)) = camera_query.single_mut() else {
        tracing::warn!(
            target: "camera",
            "[write_to_transform] Camera entity missing required components"
        );
        return;
    };

    // 写入位置
    transform.translation.x = pose.0.position.x;
    transform.translation.y = pose.0.position.y;
    transform.translation.z = Z_CAMERA;
}
