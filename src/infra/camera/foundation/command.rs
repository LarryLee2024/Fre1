//! CameraCommand — 可录制的镜头命令子集（Persistence 层）
//!
//! CameraCommand 记录了关键帧镜头操作，作为 ReplayFrame 的一部分持久化。
//! 独立于 GameCommand 的 replay 流：镜头操作是表现层行为，不影响业务逻辑确定性。
//!
//! 录制规则：
//!   - 仅录制由外部系统触发的 CameraRequest（非用户输入产生的内部请求）
//!   - 用户输入（WASD/缩放）不录制——表现层交互，不影响业务确定性
//!   - CameraBounds 设置不录制——由场景系统在回放时重新设置

use serde::{Deserialize, Serialize};

use super::target::CameraTarget;

/// 可录制的镜头命令子集——Replay 流的数据载体。
///
/// Replay 桥接为 Phase 3 实现目标，初始实现只预留序列化接口。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CameraCommand {
    /// 对应 CameraRequest::MoveTo。仅录制外部系统发起的 MoveTo。
    MoveTo(CameraTarget),

    /// 对应 CameraRequest::Follow。
    Follow(CameraTarget),

    /// 对应 CameraRequest::Unfollow。
    Unfollow,

    /// 对应 CameraRequest::SetZoom。仅录制外部系统发起的 SetZoom。
    SetZoom(f32),

    /// 对应 CameraRequest::Shake。第一个 f32 = intensity，第二个 f32 = duration。
    Shake(f32, f32),

    /// 对应 CameraRequest::Reset。
    Reset,
}
