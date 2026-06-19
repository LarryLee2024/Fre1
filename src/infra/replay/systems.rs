//! Replay Bevy Systems — 录制/回放生命周期管理
//!
//! 提供以下系统：
//! - `frame_counter_system`: PreUpdate，递增帧计数器
//! - `recording_frame_bookend_system`: PostUpdate，完成当前录制帧并开始下一帧
//! - `playback_frame_bookend_system`: PostUpdate，验证当前回放帧并推进到下一帧
//! - `rng_sync_system`: PostUpdate，将回放会话的 RNG 种子同步到资源
//!
//! 详见 ADR-041 §4-5

use bevy::prelude::*;

use crate::core::capabilities::runtime::replay::mechanism::calculate_frame_checksum;

use super::events::{ReplayCompleted, ReplayFrameProcessed, ReplayMismatchDetected};
use super::resources::{
    DeterministicRng, FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};

/// 帧计数器系统 — 每帧在 PreUpdate 递增计数器。
///
/// 帧序号从 0 开始，每次 ECS 更新周期 +1。
/// 录制时以此作为帧序号和 RNG 种子偏移。
/// 回放时用以追踪当前进度。
pub fn frame_counter_system(mut counter: ResMut<FrameCounter>) {
    counter.0 = counter.0.wrapping_add(1);
}

/// 录制帧管理（PostUpdate）— 完成当前录制帧并开始下一帧。
///
/// 每帧结束后：
/// 1. 对当前帧计算校验和并记录
/// 2. 将完成帧推入已录制列表
/// 3. 创建新的空白帧供下一周期录制
///
/// 首次调用时（start 后的第一次 PostUpdate），完成初始帧（frame 0）。
pub fn recording_frame_bookend_system(mut session: ResMut<RecordingSession>) {
    let Some(ref mut session) = session.0 else {
        return;
    };
    if !session.is_recording() {
        return;
    }

    // 使用当前帧命令计算校验和
    let checksum = session
        .recorder
        .current_frame
        .as_ref()
        .map(|f| calculate_frame_checksum(f))
        .unwrap_or(0);

    session.finalize_frame(checksum);

    // 开始下一帧（帧号 = 已完成帧数）
    let next_number = session.recorder.frame_count() as u64;
    session.start_frame(next_number, next_number);
}

/// 回放帧管理（PostUpdate）— 验证当前帧并推进到下一帧。
///
/// 每帧结束后：
/// 1. 验证当前帧校验和
/// 2. 发送 `ReplayFrameProcessed` 事件
/// 3. 推进到下一帧（更新 RNG 种子）
/// 4. 如果回放完成，切回正常模式并发送 `ReplayCompleted`
pub fn playback_frame_bookend_system(
    mut session_wrapper: ResMut<PlaybackSession>,
    mut mode_guard: ResMut<ReplayModeGuard>,
    mut commands: Commands,
) {
    let Some(ref mut session) = session_wrapper.0 else {
        return;
    };
    if !session.player.is_playing || session.is_finished() {
        return;
    }

    // 验证当前帧（不变量: 校验和一致性）
    if let Err(_e) = session.verify_current_frame() {
        let fnum = session.current_frame_number().unwrap_or(0);
        let expected = session
            .player
            .frames
            .get(fnum as usize)
            .and_then(|f| f.checksum)
            .unwrap_or(0);
        commands.trigger(ReplayMismatchDetected {
            frame: fnum,
            expected_checksum: expected,
            actual_checksum: 0, // verify_current_frame failed, actual checksum unknown
        });
    }

    // 发送帧处理事件
    if let Some(fnum) = session.current_frame_number() {
        commands.trigger(ReplayFrameProcessed {
            frame_number: fnum,
            total_frames: session.total_frames() as u64,
            commands_in_frame: session.current_commands().len(),
        });
    }

    // 推进到下一帧
    let has_more = session.advance_frame();
    if !has_more {
        let total_cmds: u64 = session
            .player
            .frames
            .iter()
            .map(|f| f.commands.len() as u64)
            .sum();
        mode_guard.0.is_replay = false;
        commands.trigger(ReplayCompleted {
            total_frames: session.total_frames() as u64,
            total_commands: total_cmds,
            has_mismatches: session.has_mismatches(),
        });
        // 回放完成后清理会话资源
        drop(session);
        session_wrapper.0 = None;
    }
}

/// RNG 种子同步（PostUpdate）— 将回放会话的 RNG 种子同步到全局资源。
///
/// 回放模式下，PlaybackSession 内部的 RNG 由日志种子驱动。
/// 游戏系统读取 `DeterministicRng` 资源获取随机数，
/// 此系统确保两者保持一致。
///
/// 应在 `playback_frame_bookend_system` 之后执行。
pub fn rng_sync_system(session: Res<PlaybackSession>, mut rng: ResMut<DeterministicRng>) {
    let Some(ref session) = session.0 else { return };
    if !session.player.is_playing || session.is_finished() {
        return;
    }

    let seeds = session.rng().get_all_seeds();
    rng.0.set_all_seeds(seeds);
}
