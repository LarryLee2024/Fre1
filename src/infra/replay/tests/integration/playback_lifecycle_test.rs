use bevy::app::App;
use bevy::prelude::*;

use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayFrame, ReplayHeader, ReplayLog,
};
use crate::core::capabilities::runtime::replay::foundation::{
    ReplayMode, ReplayModeGuard as CoreReplayModeGuard,
};
use crate::core::capabilities::runtime::replay::mechanism::{
    PlaybackSession as CorePlaybackSession, calculate_frame_checksum,
};
use crate::infra::replay::plugin::ReplayPlugin;
use crate::infra::replay::resources::{PlaybackSession, ReplayModeGuard};

/// 构建一个测试用的回放日志（2 帧，每帧 1 个命令）。
fn build_test_replay_log() -> ReplayLog {
    let header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);

    let mut frame0 = ReplayFrame::new(0, 42);
    frame0.add_command(ReplayCommand::SkipTurn {
        unit: "unit_a".to_string(),
    });
    frame0.set_checksum(calculate_frame_checksum(&frame0));

    let mut frame1 = ReplayFrame::new(1, 43);
    frame1.add_command(ReplayCommand::SkipTurn {
        unit: "unit_b".to_string(),
    });
    frame1.set_checksum(calculate_frame_checksum(&frame1));

    let mut log = ReplayLog::new(header);
    log.add_frame(frame0);
    log.add_frame(frame1);
    log.set_final_checksum(0);
    log
}

/// 回放模式：加载日志后 PlaybackSession 可逐帧推进。
#[test]
fn playback_session_advances_through_frames() {
    let log = build_test_replay_log();

    let mut core_session = CorePlaybackSession::new(ReplayMode::Full, log.header.initial_seed);
    core_session.load(&log).expect("should load valid log");
    core_session.start();

    // 验证第一帧
    let cmds = core_session.current_commands();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].type_name(), "SkipTurn");

    // 推进到第二帧
    assert!(core_session.advance_frame(), "should advance to frame 1");
    let cmds = core_session.current_commands();
    assert_eq!(cmds.len(), 1);

    // 推进到结束
    assert!(
        !core_session.advance_frame(),
        "should be finished after frame 1"
    );
    assert!(core_session.is_finished());
}

/// 回放模式：加载后帧校验和可验证。
#[test]
fn playback_session_verifies_frame_checksums() {
    let log = build_test_replay_log();

    let mut core_session = CorePlaybackSession::new(ReplayMode::Full, log.header.initial_seed);
    core_session.load(&log).expect("should load valid log");
    core_session.start();

    // 验证第一帧的校验和
    let verified = core_session
        .verify_current_frame()
        .expect("verify should not error");
    assert!(verified, "frame 0 checksum should match");

    core_session.advance_frame();

    // 验证第二帧的校验和
    let verified = core_session
        .verify_current_frame()
        .expect("verify should not error");
    assert!(verified, "frame 1 checksum should match");
}

/// 回放模式：ReplayPlugin 中 PlaybackSession Resource 可通过 ECS 访问。
#[test]
fn playback_session_is_accessible_as_resource() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let log = build_test_replay_log();
    let mut core_session = CorePlaybackSession::new(ReplayMode::Full, log.header.initial_seed);
    core_session.load(&log).expect("should load");
    core_session.start();

    app.world_mut()
        .insert_resource(PlaybackSession(Some(core_session)));

    // 设置回放模式
    let mut guard = app.world_mut().resource_mut::<ReplayModeGuard>();
    guard.0 = CoreReplayModeGuard::replay_mode();

    // PlaybackSession Resource 可通过 Res 访问
    let session = app.world().resource::<PlaybackSession>();
    assert!(session.0.is_some(), "playback session should be present");
    if let Some(ref s) = session.0 {
        assert!(s.player.is_playing, "should be playing");
        assert_eq!(s.total_frames(), 2, "should have 2 frames");
    }
}

/// 回放模式完成后 ReplayModeGuard 应切回正常模式。
#[test]
fn playback_completion_resets_mode_guard() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ReplayPlugin);

    let log = build_test_replay_log();
    let mut core_session = CorePlaybackSession::new(ReplayMode::Full, log.header.initial_seed);
    core_session.load(&log).expect("should load");
    core_session.start();

    // 标记回放模式
    {
        let mut guard = app.world_mut().resource_mut::<ReplayModeGuard>();
        guard.0.is_replay = true;
    }

    app.world_mut()
        .insert_resource(PlaybackSession(Some(core_session)));

    // 运行帧让 playback_frame_bookend_system 推进回放
    app.update();
    app.update();
    app.update(); // 第三帧：回放应已完成

    // 验证：回放完成后 mode_guard 应切回正常
    let guard = app.world().resource::<ReplayModeGuard>();
    assert!(
        !guard.0.is_replay,
        "mode guard should reset after playback completes"
    );

    // 验证：PlaybackSession 应变为 None
    let session = app.world().resource::<PlaybackSession>();
    assert!(
        session.0.is_none(),
        "playback session should be None after completion"
    );
}
