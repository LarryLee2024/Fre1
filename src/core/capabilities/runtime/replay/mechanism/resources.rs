//! Replay ECS Resources — 将核心回放类型包装为 ECS Resource
//!
//! 提供三个层面的资源：
//! - 横切共享资源：ReplayModeGuard（回放模式标记）
//! - 会话资源：RecordingSession（录制中）、PlaybackSession（回放中），None 表示不在对应模式
//! - 辅助资源：FrameCounter（帧计数器，从 0 开始递增）
//!
//! 详见 ADR-041 §6 和 replay_schema.md

use bevy::prelude::*;

use crate::core::capabilities::runtime::replay::foundation::ReplayModeGuard as CoreReplayModeGuard;
use crate::core::capabilities::runtime::replay::mechanism::{
    PlaybackSession as CorePlaybackSession, RecordingSession as CoreRecordingSession,
};

/// 回放模式守卫 — 标记当前是否处于回放模式。
///
/// 回放模式下所有读取外部状态的操作（系统时间、文件系统、非确定性 RNG）均需被禁止。
/// 各系统应通过 `guard.0.is_replay` 判断当前模式并采取对应行为。
///
/// 详见 ADR-041 §6
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[derive(Default)]
pub struct ReplayModeGuard(pub CoreReplayModeGuard);

/// 录制会话资源 — Some 表示正在录制，None 表示不在录制模式。
///
/// 外部系统通过 `RecordingSession::start()` 初始化并开始录制，
/// 通过 `RecordingSession::stop()` 结束录制并获取 ReplayLog。
/// `recording_frame_bookend_system` 自动处理帧边界。
#[derive(Resource, Default)]
pub struct RecordingSession(pub Option<CoreRecordingSession>);

/// 回放会话资源 — Some 表示正在回放，None 表示不在回放模式。
///
/// 外部系统通过 `PlaybackSession::load()` + `PlaybackSession::start()` 初始化，
/// `playback_frame_bookend_system` 自动处理帧推进。
/// 各业务系统通过 `current_commands()` 读取当前帧命令。
#[derive(Resource, Default)]
pub struct PlaybackSession(pub Option<CorePlaybackSession>);

/// 帧计数器 — 从 0 开始，每帧递增 1。
///
/// 用于录制时计算帧序号，和回放时追踪进度。
/// 帧序号对应游戏逻辑更新周期，而非渲染帧率。
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[derive(Default)]
pub struct FrameCounter(pub u64);
