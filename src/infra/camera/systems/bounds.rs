//! Bounds — 镜头边界约束
//!
//! 在 PostUpdate 中调度（Interpolate 之后、Shake 之前）。
//! 读取 CameraBounds Component，执行 position 钳位。
//!
//! Camera 不感知 Map Domain / Terrain Domain 的任何类型。
//! CameraBounds 使用纯 Vec2，不包含 GridPos/TileMap 引用。
//! 边界数据由场景系统在 OnEnter 时以 Vec2 形式注入。

use bevy::prelude::*;

use crate::infra::camera::components::{CameraBounds, MainCamera};
use crate::infra::camera::foundation::pose::CurrentPose;

/// 应用摄像机边界钳位——由 CameraPlugin 在 PostUpdate 中注册。
///
/// 执行规则：
///   1. 检查 Camera Entity 是否有 CameraBounds Component
///   2. 如果有：pose.position = pose.position.clamp(bounds.min, bounds.max)
///   3. 如果没有：跳过钳位，允许镜头移动到任何位置
///   4. 如果 bounds.min > bounds.max：视为配置错误，跳过钳位并记录警告
pub fn clamp_position(
    mut camera_query: Query<(&mut CurrentPose, Option<&CameraBounds>), With<MainCamera>>,
) {
    let Ok((mut pose, bounds)) = camera_query.single_mut() else {
        return;
    };

    let Some(bounds) = bounds else {
        return; // 没有边界约束，不执行钳位
    };

    // 检查边界合法性
    if bounds.min.x > bounds.max.x || bounds.min.y > bounds.max.y {
        tracing::warn!(
            target: "camera",
            "[clamp_position] CameraBounds min > max: min={:?} max={:?}, skipping clamp",
            bounds.min,
            bounds.max,
        );
        return;
    }

    // 执行钳位
    pose.0.position = pose.0.position.clamp(bounds.min, bounds.max);
}
