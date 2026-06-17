//! ReplayPlugin — 回放基础设施 ECS Plugin
//!
//! 注册回放相关的 Resource 和 System。
//! 事件（observer-based）无需显式注册，触发时自动注册。
//!
//! ## 注册内容
//!
//! **Resources**:
//! - `DeterministicRng` — 全局确定性 RNG（FromWorld，种子 = 0）
//! - `ReplayModeGuard` — 回放模式标记（默认正常模式）
//! - `RecordingSession` — 录制会话（None）
//! - `PlaybackSession` — 回放会话（None）
//! - `FrameCounter` — 帧计数器（0）
//!
//! **Systems**:
//! - `PreUpdate`: `frame_counter_system`（递增帧号）
//! - `PostUpdate`: `recording_frame_bookend_system`（录制帧边界）
//! - `PostUpdate`: `playback_frame_bookend_system`（回放帧推进）
//! - `PostUpdate`: `rng_sync_system`（RNG 种子同步，在 playback 之后，.chain()）
//!
//! ## 注意
//!
//! - RecordingSession 和 PlaybackSession 互斥，外部系统应保证不会同时激活。
//! - 录制/回放的实际开始/停止由业务系统（如 combat 或 battle）通过
//!   操作 Session Resource 触发，本 Plugin 仅提供帧级的自动管理。
//! - 回放帧命令的分发由各业务系统自行实现（读取 `PlaybackSession.current_commands()`）。
//!
//! 详见 ADR-041 §4-5
//! 详见 replay_schema.md §5

use bevy::prelude::*;

use super::systems::{
    frame_counter_system, playback_frame_bookend_system, recording_frame_bookend_system,
    rng_sync_system,
};

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        // ── Resources ──
        app.init_resource::<super::resources::DeterministicRng>();
        app.init_resource::<super::resources::ReplayModeGuard>();
        app.init_resource::<super::resources::RecordingSession>();
        app.init_resource::<super::resources::PlaybackSession>();
        app.init_resource::<super::resources::FrameCounter>();

        // ── Systems ──
        // PreUpdate: 帧计数器自增
        app.add_systems(PreUpdate, frame_counter_system);

        // PostUpdate: 录制/回放帧管理，rng_sync 最后执行以确保种子就绪
        app.add_systems(
            PostUpdate,
            (
                recording_frame_bookend_system,
                playback_frame_bookend_system,
                rng_sync_system,
            )
                .chain(),
        );

        tracing::info!("[ReplayPlugin] initialized (resources, frame systems)");
    }
}
