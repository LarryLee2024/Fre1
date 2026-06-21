//! CameraPose — 镜头姿态值对象
//!
//! 包含 position (Vec2)、zoom (f32)、rotation (f32)。
//! 零 ECS 依赖，可独立测试。
//!
//! 运行时在 Camera Entity 上以两个独立 Component 存在：
//! - TargetPose：状态机设置的目标姿态
//! - CurrentPose：每秒插值逼近 TargetPose 的当前姿态

use bevy::prelude::Vec2;
use bevy::prelude::*;

/// 镜头姿态值对象。
///
/// 表示镜头在二维世界空间中的完整姿态——位置、缩放、旋转。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct CameraPose {
    /// 世界坐标位置（二维）。范围受 CameraBounds 约束（如果设置）。
    pub position: Vec2,

    /// 缩放倍数。1.0 = 默认缩放。范围 [0.5, 3.0]。
    pub zoom: f32,

    /// 旋转角度（弧度）。当前阶段始终为 0，预留字段。
    pub rotation: f32,
}

impl CameraPose {
    /// 对目标姿态执行帧率无关的线性插值。
    ///
    /// t 的范围 [0.0, 1.0] —— 0.0 = 完全当前姿态，1.0 = 完全目标姿态。
    /// 插值是确定性数学运算，不依赖随机数。
    pub fn lerp(&self, target: &CameraPose, t: f32) -> CameraPose {
        CameraPose {
            position: self.position.lerp(target.position, t),
            zoom: self.zoom + (target.zoom - self.zoom) * t,
            rotation: self.rotation + (target.rotation - self.rotation) * t,
        }
    }
}

impl Default for CameraPose {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

// ─── TargetPose / CurrentPose Component 包装 ─────────────────────

use bevy::prelude::Component;

/// TargetPose — 目标姿态 Component，挂在 Camera Entity 上。
///
/// 由状态机/input_handler 设置，描述镜头最终要到达的位姿。
/// movement 系统读取此值并插值更新 CurrentPose。
#[derive(Component, Debug, Clone, PartialEq, Default, Reflect)]
pub struct TargetPose(pub CameraPose);

/// CurrentPose — 当前姿态 Component，挂在 Camera Entity 上。
///
/// 每帧由 movement 系统插值逼近 TargetPose。
/// 写入 Transform 前会经过 CameraBounds 钳位和震屏偏移叠加。
#[derive(Component, Debug, Clone, PartialEq, Default, Reflect)]
pub struct CurrentPose(pub CameraPose);

// ─── 常量 ─────────────────────────────────────────────────────

/// 缩放最小值
pub const MIN_ZOOM: f32 = 0.5;
/// 缩放最大值
pub const MAX_ZOOM: f32 = 3.0;
/// 缩放步进系数（用户每按一次缩放的变化比例）
pub const ZOOM_STEP_FACTOR: f32 = 1.5;
/// 自由移动速度（像素/秒）
pub const CAMERA_MOVE_SPEED: f32 = 500.0;
/// 插值速度系数（越大则跟随越快）
pub const LERP_SPEED: f32 = 8.0;
/// 震屏振荡频率（Hz）
pub const SHAKE_FREQUENCY: f32 = 10.0;
/// FreeMove 空闲超时（秒）
pub const FREE_MOVE_TIMEOUT: f32 = 2.0;
/// Camera Z 深度
pub const Z_CAMERA: f32 = 10.0;
