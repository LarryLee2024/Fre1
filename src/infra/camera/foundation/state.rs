//! CameraState — 镜头状态机枚举
//!
//! 所有镜头行为通过此状态机仲裁，禁止外部系统直接修改 Transform。
//! 同一时刻有且仅有一个活跃的 CameraState 值。

use bevy::prelude::*;

use super::target::CameraTarget;

/// 镜头状态机——所有镜头行为通过此状态机仲裁。
///
/// 挂在 Camera Entity 上作为 ECS Component。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub enum CameraState {
    /// 空闲：未触发任何主动行为，镜头静止。
    /// 默认状态。进入动作：设置 TargetPose = CurrentPose（停止移动）。
    Idle,

    /// 自由移动：玩家通过 WASD/方向键控制镜头。
    /// 进入动作：启动 IdleTimeout 计时器（2 秒）。
    /// 输入停止 2 秒自动回到 Idle。
    FreeMove,

    /// 跟随：镜头跟随一个 CameraTarget（单位移动等）。
    /// 用户输入时切换为 FreeMove。
    Follow(CameraTarget),

    /// 聚焦：镜头动画过渡到特定 CameraTarget。
    /// 进入动作：设置 TargetPose = 目标位置，锁定输入。
    /// 聚焦期间不处理新的外部 CameraRequest（初始实现）。
    /// elapsed >= duration 时自动回到 Idle。
    Focus {
        /// 聚焦目标
        target: CameraTarget,
        /// 聚焦动画总时长（秒）
        duration: f32,
        /// 已过时间（秒），每帧由 state_machine system 推进
        elapsed: f32,
    },
}
