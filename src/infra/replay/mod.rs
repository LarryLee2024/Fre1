//! replay — 基础设施回放层
//!
//! 将 Core 层的回放能力（录制器、播放器、确定性 RNG、校验验证器）
//! 桥接到 Bevy ECS，提供 Resource 包装、System 自动管理和 Plugin 注册。
//!
//! ## 模块结构
//!
//! | 模块 | 职责 |
//! |------|------|
//! | `resources.rs` | Bevy Resource 包装（DeterministicRng, ReplayModeGuard, 会话资源） |
//! | `systems.rs`   | 录制/回放生命周期 System（帧管理、RNG 同步） |
//! | `events.rs`    | Infra 层回放事件 |
//! | `plugin.rs`    | ReplayPlugin 注册入口 |
//!
//! ## 使用示例
//!
//! ```ignore
//! // 录制模式
//! fn start_recording(mut session: ResMut<RecordingSession>, mut rng: ResMut<DeterministicRng>) {
//!     let header = ReplayHeader::new(1, "0.1.0", "battle_scene_001", 42);
//!     let mut core_session = CoreRecordingSession::new(60);
//!     core_session.start(header, 42);
//!     session.0 = Some(core_session);
//!     rng.0 = DeterministicRng::with_seed(42);
//! }
//!
//! // 回放模式
//! fn start_playback(mut session: ResMut<PlaybackSession>, log: ReplayLog) {
//!     let mut core_session = CorePlaybackSession::new(ReplayMode::Full, log.header.initial_seed);
//!     core_session.load(&log).unwrap();
//!     core_session.start();
//!     session.0 = Some(core_session);
//! }
//! ```
//!
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md`
//! 详见 `docs/04-data/infrastructure/replay_schema.md`

mod plugin;
pub(crate) mod resources;
pub(crate) mod systems;

mod events;

#[cfg(test)]
mod tests;

// ── 核心类型 re-export ──
pub use crate::core::capabilities::runtime::replay::foundation::{
    AbilityTarget, ReplayCommand, ReplayError, ReplayFrame, ReplayHeader, ReplayLog,
    ReplayMismatch, ReplayMode, RngSeeds, RngStream,
};

// ── Infra 资源 re-export ──
pub use resources::{
    DeterministicRng, FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};

// ── 事件 re-export ──
pub use events::{
    RecordingCompleted, RecordingStarted, ReplayCompleted, ReplayFrameProcessed,
    ReplayMismatchDetected, ReplayStarted,
};

// ── 机制 re-export ──
pub use crate::core::capabilities::runtime::replay::mechanism::{
    PlaybackSession as CorePlaybackSession, RecordingSession as CoreRecordingSession,
    calculate_frame_checksum, fast_forward, validate_frame_sequence, validate_version,
};

pub use plugin::*;
