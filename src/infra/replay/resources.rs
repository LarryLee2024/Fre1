//! Replay Bevy Resources — 将核心回放类型包装为 ECS Resource
//!
//! 提供三个层面的资源：
//! - 横切共享资源：DeterministicRng（所有随机操作入口）、ReplayModeGuard（回放模式标记）
//! - 会话资源：RecordingSession（录制中）、PlaybackSession（回放中），None 表示不在对应模式
//! - 辅助资源：FrameCounter（帧计数器，从 0 开始递增）
//!
//! ECS 包装器已迁移至 `core::capabilities::runtime::replay::mechanism::resources`，
//! 本模块仅做 re-export 以保持向后兼容。
//!
//! 详见 ADR-041 §6 和 replay_schema.md

pub use crate::shared::random::DeterministicRng;
pub use crate::core::capabilities::runtime::replay::mechanism::resources::{
    FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};
