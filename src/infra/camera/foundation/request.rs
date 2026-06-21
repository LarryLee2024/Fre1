//! CameraRequest — 所有镜头请求的统一枚举
//!
//! 外部系统通过 `commands.trigger(CameraRequest::...)` 发送此事件。
//! CameraPlugin 内的 Observer 消费此事件并更新状态机。
//! 禁止外部系统直接修改 Camera Entity 的 Transform/Projection。

use bevy::prelude::*;

use super::target::CameraTarget;

/// 所有镜头请求的统一枚举——Camera 系统的唯一外部修改入口。
///
/// 此类型的本质是 Event，在 trigger 时瞬时存在，不持久化。
/// 可录制的请求同时以 CameraCommand 形式录制到 Replay 流。
#[derive(Event, Debug, Clone, PartialEq)]
pub enum CameraRequest {
    /// 移动到指定世界位置。
    ///
    /// target: 目标位置（WorldPos/TilePos/UnitId）
    /// duration: 插值过渡时长（秒）。0 = 瞬移。>= 0，负值视为 0。
    MoveTo { target: CameraTarget, duration: f32 },

    /// 跟随一个目标。
    ///
    /// target: 跟随目标（UnitId/TilePos/WorldPos）
    /// 约束：Focus 状态下静默忽略。
    Follow { target: CameraTarget },

    /// 取消跟随。非 Follow 状态下静默忽略。
    Unfollow,

    /// 设置缩放级别。
    ///
    /// zoom: 目标缩放倍数 [0.5, 3.0]，超出自动钳位
    /// duration: 过渡时长（秒）
    SetZoom { zoom: f32, duration: f32 },

    /// 震屏效果。
    ///
    /// intensity: 震动强度 [1.0, 20.0]，超出自动钳位
    /// duration: 震动时长（秒）[0, 5.0]，超出自动钳位，0 静默忽略
    /// 确定性：震屏偏移使用确定性算法，确保 Replay 一致。
    Shake { intensity: f32, duration: f32 },

    /// 重置镜头到默认位置。
    ///
    /// duration: 过渡时长（秒）。0 = 瞬移。
    Reset { duration: f32 },

    /// 锁定用户镜头控制（FreeMove 输入）。
    LockInput,

    /// 解锁用户镜头控制。
    UnlockInput,
}
