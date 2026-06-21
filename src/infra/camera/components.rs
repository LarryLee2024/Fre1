//! Camera Components — ECS Component 类型（不包含业务规则）
//!
//! 类型分布在 foundation/ 和本文件中。Foundation 包含核心值类型，
//! 本文件包含 ECS 专属 Component 类型。

use bevy::prelude::*;

// ─── MainCamera 标记 ─────────────────────────────────────────

/// 主摄像机标记组件——用于外部系统识别主镜头。
///
/// CameraPlugin 在 spawn 时插入此标记。
/// 任何系统中查询 Camera 时应使用此标记过滤。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub struct MainCamera;

// ─── CameraBounds ────────────────────────────────────────────

/// 镜头边界约束——限制镜头在世界空间中的移动范围。
///
/// 挂在 Camera Entity 上作为 ECS Component。
/// 由场景系统在 OnEnter 时插入（根据 GridMap 尺寸计算边界），
/// Camera 系统只读取不创建/修改。
///
/// 边界不存在时的行为：不执行钳位，镜头可自由移动。
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
pub struct CameraBounds {
    /// 边界最小值（世界坐标，左下角）
    pub min: Vec2,
    /// 边界最大值（世界坐标，右上角）
    pub max: Vec2,
}

// ─── CameraShake ─────────────────────────────────────────────

/// 震屏效果状态——临时 Component，震屏期间存在于 Camera Entity 上。
///
/// 由 CameraRequest::Shake 触发时创建，震屏结束时自动移除。
/// 震屏偏移在 ClampSystem 之后、TransformWrite 之前叠加到位置。
#[derive(Component, Clone, Debug, Reflect)]
pub struct CameraShake {
    /// 震动强度 [1.0, 20.0]
    pub intensity: f32,
    /// 震动总时长（秒）[0, 5.0]
    pub duration: f32,
    /// 已过时间（秒）
    pub elapsed: f32,
    /// 当前帧的偏移量（每帧更新）
    pub current_offset: Vec2,
    /// 确定性 RNG 种子偏移（Phase 3 启用，当前为 0）
    pub seed_offset: u64,
}

impl CameraShake {
    /// 创建新的震屏状态。
    pub fn new(intensity: f32, duration: f32, seed_offset: u64) -> Self {
        Self {
            intensity: intensity.clamp(1.0, 20.0),
            duration: duration.clamp(0.0, 5.0),
            elapsed: 0.0,
            current_offset: Vec2::ZERO,
            seed_offset,
        }
    }
}

// ─── CameraInputBlock ────────────────────────────────────────

/// 输入阻塞堆叠计数器——支持多个系统同时锁定用户镜头控制。
///
/// 工作原理：
///   - CameraRequest::LockInput → block_count += 1
///   - CameraRequest::UnlockInput → block_count = saturating_sub(1)
///   - block_count > 0 时，所有 FreeMove 相关输入被忽略
///   - block_count == 0 时，正常响应 FreeMove 输入
#[derive(Component, Default, Debug, Clone, PartialEq, Reflect)]
pub struct CameraInputBlock {
    /// 当前阻塞计数。0 = 输入正常。>0 = 输入被阻塞。
    pub block_count: u32,
}

// ─── IdleTimeout ─────────────────────────────────────────────

/// FreeMove 空闲超时计时器。
///
/// 仅在 CameraState == FreeMove 时存在。
/// 输入停止 timeout_duration 秒后自动回到 Idle。
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
pub struct IdleTimeout {
    /// 从上一次用户输入开始的已过时间（秒）
    pub elapsed: f32,
    /// 超时时长（秒）。默认值：2.0 秒。
    pub timeout_duration: f32,
}

impl Default for IdleTimeout {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            timeout_duration: 2.0,
        }
    }
}
