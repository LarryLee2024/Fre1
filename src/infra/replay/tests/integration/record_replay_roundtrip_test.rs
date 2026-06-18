//! 端到端录制-回放一致性测试
//!
//! 验证核心不变量：录制命令 → 构建 ReplayLog → 回放 → 命令逐一一致。
//! 来源：docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md

use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayFrame, ReplayHeader, ReplayLog, ReplayMode,
};
use crate::core::capabilities::runtime::replay::mechanism::{
    PlaybackSession, calculate_frame_checksum,
};

/// 辅助：构建一个包含测试命令的 ReplayLog。
fn make_test_log(frame_count: usize, commands_per_frame: usize) -> ReplayLog {
    let mut header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);

    let mut frames = Vec::new();
    for i in 0..frame_count {
        let mut frame = ReplayFrame::new(i as u64, i as u64);
        for j in 0..commands_per_frame {
            frame.add_command(ReplayCommand::UnitMove {
                unit: format!("unit_{}", j),
                path: vec![format!("pos_{}_{}", i, j)],
            });
        }
        frame.set_checksum(calculate_frame_checksum(&frame));
        frames.push(frame);
    }

    header.set_total_frames(frames.len() as u64);

    ReplayLog {
        header,
        frames,
        final_checksum: None,
    }
}

/// 端到端：录制 3 帧 × 2 命令，回放后命令逐一一致。
#[test]
fn record_then_replay_command_consistency() {
    let log = make_test_log(3, 2);
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).expect("load should succeed");
    session.start();

    // 逐帧验证命令内容
    for (frame_idx, expected_frame) in log.frames.iter().enumerate() {
        assert!(
            !session.is_finished(),
            "should not be finished at frame {}",
            frame_idx
        );

        let actual_commands = session.current_commands();
        assert_eq!(
            actual_commands.len(),
            expected_frame.commands.len(),
            "command count mismatch at frame {}",
            frame_idx
        );

        for (cmd_idx, (actual, expected)) in actual_commands
            .iter()
            .zip(expected_frame.commands.iter())
            .enumerate()
        {
            assert_eq!(
                format!("{:?}", actual),
                format!("{:?}", expected),
                "command mismatch at frame {} cmd {}",
                frame_idx,
                cmd_idx
            );
        }

        session.advance_frame();
    }

    assert!(session.is_finished(), "session should be finished");
}

/// 不变量：回放帧校验和验证通过（无篡改）。
#[test]
fn replay_frame_checksum_verify_pass() {
    let log = make_test_log(2, 1);
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).expect("load should succeed");
    session.start();

    // 逐帧验证校验和
    for _ in 0..log.frames.len() {
        let result = session.verify_current_frame();
        assert!(result.is_ok(), "checksum verification should pass");
        session.advance_frame();
    }
}

/// 不变量：空帧回放不崩溃。
#[test]
fn replay_empty_frame_no_panic() {
    let mut frame = ReplayFrame::new(0, 0);
    frame.set_checksum(calculate_frame_checksum(&frame));

    let header = ReplayHeader::new(1, "0.1.0", "test_scene", 42);

    let log = ReplayLog {
        header,
        frames: vec![frame],
        final_checksum: None,
    };

    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).expect("load should succeed");
    session.start();

    let result = session.verify_current_frame();
    assert!(result.is_ok());
    assert!(session.current_commands().is_empty());
}

/// 不变量：不同帧有不同命令。
#[test]
fn different_frames_different_commands() {
    let log = make_test_log(3, 1);

    // 确认每帧命令不同（因为 path 不同）
    for i in 0..log.frames.len() {
        for j in (i + 1)..log.frames.len() {
            assert_ne!(
                format!("{:?}", log.frames[i].commands),
                format!("{:?}", log.frames[j].commands),
                "frames {} and {} should have different commands",
                i,
                j
            );
        }
    }
}
