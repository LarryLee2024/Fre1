//! 回放会话 RNG 帧对齐确定性测试（Replay → RNG 确定性）
//!
//! 验证 PlaybackSession 在逐帧推进时正确管理 RNG 状态：
//! - 相同日志 + 相同种子 → 帧推进后 RNG 输出一致
//! - 两独立会话相同输入 → 每帧 RNG 输出一致
//! - 帧边界重置后 RNG 种子正确恢复
//!
//! 领域规则来源：ADR-041 §3（确定性 RNG 种子同步）

use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayFrame, ReplayHeader, ReplayLog, ReplayMode,
};
use crate::core::capabilities::runtime::replay::mechanism::{
    PlaybackSession as CorePlaybackSession, calculate_frame_checksum,
};
use crate::shared::random::{RngSeeds, RngStream};

/// 辅助：构造多帧 ReplayLog。
/// 每帧包含指定数量的命令，使用帧号作为 rng_seed_offset。
fn make_multi_frame_log(
    frame_count: usize,
    commands_per_frame: usize,
    initial_seed: u64,
) -> ReplayLog {
    let header = ReplayHeader::new(1, "0.1.0", "rng_test", initial_seed);
    let mut frames = Vec::new();

    for i in 0..frame_count {
        let mut frame = ReplayFrame::new(i as u64, i as u64);
        for j in 0..commands_per_frame {
            frame.add_command(ReplayCommand::SkipTurn {
                unit: format!("unit_{}_{}", i, j),
            });
        }
        frame.set_checksum(calculate_frame_checksum(&frame));
        frames.push(frame);
    }

    ReplayLog {
        header,
        frames,
        final_checksum: None,
    }
}

/// 相同 PlaybackSession 在不同帧上 RNG 输出确定性。
///
/// Given: 相同初始种子 + 相同 ReplayLog
/// When: 逐帧推进
/// Then: 每帧 RNG 输出序列与帧号相关（不同帧不同种子偏移）
#[test]
fn playback_session_rng_advances_per_frame() {
    let log = make_multi_frame_log(3, 2, 42);
    let mut session = CorePlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).expect("load");
    session.start();

    // 记录各帧的 RNG 状态
    let mut per_frame_seeds = Vec::new();

    while !session.is_finished() {
        let seeds = session.rng().get_all_seeds();
        per_frame_seeds.push(seeds);
        session.advance_frame();
    }

    // 不同帧之间种子应不同（因 rng_seed_offset 不同）
    assert_eq!(per_frame_seeds.len(), 3, "should have 3 frame seed states");
    assert_ne!(
        per_frame_seeds[0], per_frame_seeds[1],
        "frame 0 and frame 1 seeds should differ (different rng_seed_offset)"
    );
    assert_ne!(
        per_frame_seeds[1], per_frame_seeds[2],
        "frame 1 and frame 2 seeds should differ"
    );
}

/// 两独立 PlaybackSession 相同输入 → 每帧 RNG 输出一致。
///
/// Given: 两个相同的 PlaybackSession（相同种子 + 相同日志）
/// When: 逐帧推进并采样 RNG 输出
/// Then: 每帧两会话的 RNG 输出完全相同
#[test]
fn two_sessions_same_input_same_rng_output() {
    let log = make_multi_frame_log(4, 1, 42);

    let mut session_a = CorePlaybackSession::new(ReplayMode::Full, 42);
    session_a.load(&log).expect("load a");
    session_a.start();

    let mut session_b = CorePlaybackSession::new(ReplayMode::Full, 42);
    session_b.load(&log).expect("load b");
    session_b.start();

    for frame_idx in 0..4 {
        // 采样两会话的 RNG 输出
        let a_combat = session_a.rng_mut().next_u64(RngStream::Combat);
        let a_drop = session_a.rng_mut().next_u64(RngStream::Drop);
        let a_ai = session_a.rng_mut().next_u64(RngStream::AI);
        let a_world = session_a.rng_mut().next_u64(RngStream::World);

        let b_combat = session_b.rng_mut().next_u64(RngStream::Combat);
        let b_drop = session_b.rng_mut().next_u64(RngStream::Drop);
        let b_ai = session_b.rng_mut().next_u64(RngStream::AI);
        let b_world = session_b.rng_mut().next_u64(RngStream::World);

        assert_eq!(
            a_combat, b_combat,
            "frame {}: RNG Combat output must match between sessions",
            frame_idx
        );
        assert_eq!(
            a_drop, b_drop,
            "frame {}: RNG Drop output must match",
            frame_idx
        );
        assert_eq!(a_ai, b_ai, "frame {}: RNG AI output must match", frame_idx);
        assert_eq!(
            a_world, b_world,
            "frame {}: RNG World output must match",
            frame_idx
        );

        session_a.advance_frame();
        session_b.advance_frame();
    }
}

/// 回放完成后 RNG 种子固定（再次 start 会重置）。
///
/// Given: 一个已完成的 PlaybackSession
/// When: 重新 load + start
/// Then: RNG 状态恢复到初始状态
#[test]
fn playback_session_rng_reset_on_restart() {
    let log = make_multi_frame_log(2, 1, 42);
    let mut session = CorePlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).expect("first load");
    session.start();

    // 消耗一些 RNG
    let first_val = session.rng_mut().next_u64(RngStream::Combat);
    session.advance_frame();
    assert!(!session.is_finished(), "should have more frames");
    session.advance_frame();
    assert!(session.is_finished(), "should be finished");

    // 重新加载并开始
    session.load(&log).expect("second load");
    session.start();

    // 再次读取 RNG，应与首次一致（确定性）
    let restarted_val = session.rng_mut().next_u64(RngStream::Combat);
    assert_eq!(
        first_val, restarted_val,
        "RNG should reset to initial state on restart"
    );
}

/// PlaybackSession 首次 start 前的 RNG 状态不应影响回放。
///
/// Given: 预修改 RNG 状态的 PlaybackSession
/// When: load + start 后
/// Then: RNG 状态被正确覆盖（不残留自定义设置）
#[test]
fn rng_state_overwritten_by_load() {
    let log = make_multi_frame_log(1, 1, 99);

    // 先修改 RNG 种子
    let mut session = CorePlaybackSession::new(ReplayMode::Full, 42);
    let custom_seeds = RngSeeds::new(111, 222, 333, 444);
    session.rng_mut().set_all_seeds(custom_seeds);

    // load + start 应覆盖 RNG 状态
    session.load(&log).expect("load");
    session.start();

    // 验证 RNG 已被日志的种子覆盖（initial_seed=99）
    let seeds = session.rng().get_all_seeds();
    assert_eq!(
        seeds.combat_seed,
        RngSeeds::uniform(99).combat_seed,
        "RNG seeds should be overwritten by loaded log's initial seed"
    );
}

/// 帧间 RNG 流的隔离性测试：
/// 不同流在同一帧内使用互不干扰。
///
/// Given: 一个 PlaybackSession
/// When: 在同一帧内依次调用不同流
/// Then: 各流独立（Combat 不影响 Drop，AI 不影响 World）
#[test]
fn rng_streams_are_independent_within_frame() {
    let log = make_multi_frame_log(1, 0, 42);
    let mut session = CorePlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).expect("load");
    session.start();

    // 在同一帧内连续调用各流
    let c1 = session.rng_mut().next_u64(RngStream::Combat);
    let d1 = session.rng_mut().next_u64(RngStream::Drop);
    let a1 = session.rng_mut().next_u64(RngStream::AI);
    let w1 = session.rng_mut().next_u64(RngStream::World);

    // 各流的产生概率极低（各 ChaCha12 独立）
    // 仅验证四个值不全部相等（那将是极不可能的碰撞）
    let all_same = c1 == d1 && d1 == a1 && a1 == w1;
    assert!(
        !all_same,
        "streams should produce different values (independent ChaCha12 instances)"
    );
}
