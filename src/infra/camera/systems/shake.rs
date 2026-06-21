//! Shake — 震屏效果
//!
//! 在 PostUpdate 中调度（ClampSystem 之后、TransformWrite 之前）。
//! 消费 CameraRequest::Shake 触发的 CameraShake Component，生成确定性震屏偏移。
//!
//! 确定性要求：震屏偏移使用确定性数学公式（基于 elapsed 和 seed_offset），
//! 确保 Replay 回放时震屏结果一致。

use bevy::prelude::*;

use crate::infra::camera::components::{CameraShake, MainCamera};
use crate::infra::camera::foundation::pose::{CurrentPose, SHAKE_FREQUENCY};

/// 应用震屏偏移——由 CameraPlugin 在 PostUpdate 中注册。
///
/// 每帧检查 CameraShake Component：
///   1. 如果存在且在计时器有效期内，计算确定性偏移并叠加到 CurrentPose.position
///   2. 偏移量随 elapsed 线性衰减
///   3. 计时器到期后移除 CameraShake Component
///
/// 震屏偏移算法：
///   offset_magnitude = intensity * (1.0 - elapsed/duration)  // 线性衰减
///   angle = elapsed * TAU * SHAKE_FREQUENCY
///   offset = Vec2(cos(angle), sin(angle)) * magnitude
///
/// 这是确定性算法——同一种子 + 相同的 elapsed 产生完全相同的偏移。
pub fn apply_shake(
    time: Res<Time>,
    mut camera_query: Query<(Entity, &mut CurrentPose, &mut CameraShake), With<MainCamera>>,
    mut commands: Commands,
) {
    let Ok((entity, mut pose, mut shake)) = camera_query.single_mut() else {
        return;
    };

    // 推进计时
    shake.elapsed += time.delta().as_secs_f32();

    // 震屏结束 → 移除 Component
    if shake.elapsed >= shake.duration {
        commands.entity(entity).remove::<CameraShake>();
        return;
    }

    // 计算震屏偏移（确定性算法）
    let decay = 1.0 - (shake.elapsed / shake.duration); // 线性衰减 [1.0 → 0.0]
    let magnitude = shake.intensity * decay;

    // 使用 elapsed * frequency 作为角度，产生确定性振荡
    let angle = shake.elapsed * std::f32::consts::TAU * SHAKE_FREQUENCY;
    shake.current_offset = Vec2::new(angle.cos(), angle.sin()) * magnitude;

    // 叠加到 CurrentPose.position
    pose.0.position += shake.current_offset;
}
