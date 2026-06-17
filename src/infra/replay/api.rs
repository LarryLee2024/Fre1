//! Replay Public API — 对外统一出口
//!
//! 所有需要回放能力的模块通过此模块访问回放功能。
//! 遵循 "pub(crate) impl, pub api" 原则：
//! - 基础设施实现对外不可见
//! - 仅通过此 API 暴露必要的类型和系统

// ── 核心类型 re-export ──
pub use crate::core::capabilities::runtime::replay::foundation::{
    AbilityTarget, ReplayCommand, ReplayError, ReplayFrame, ReplayHeader, ReplayLog,
    ReplayMismatch, ReplayMode, RngSeeds, RngStream,
};

// ── Infra 资源 re-export ──
pub use super::resources::{
    DeterministicRng, FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};

// ── 事件 re-export ──
pub use super::events::{
    RecordingCompleted, RecordingStarted, ReplayCompleted, ReplayFrameProcessed,
    ReplayMismatchDetected, ReplayStarted,
};

// ── 机制 re-export ──
pub use crate::core::capabilities::runtime::replay::mechanism::{
    PlaybackSession as CorePlaybackSession, RecordingSession as CoreRecordingSession,
    calculate_frame_checksum, fast_forward, validate_frame_sequence, validate_version,
};
