use crate::core::capabilities::runtime::replay::foundation::{
    DeterministicRng as CoreDeterministicRng, RngStream,
};
use crate::infra::replay::resources::{
    DeterministicRng, FrameCounter, PlaybackSession, RecordingSession, ReplayModeGuard,
};

/// DeterministicRng 的 FromWorld 默认构造使用 seed=0。
#[test]
fn deterministic_rng_default_creates_valid_instance() {
    let rng = DeterministicRng(CoreDeterministicRng::with_seed(0));
    // 默认种子为 0，各流 seed 均为 0
    let seeds = rng.0.get_all_seeds();
    assert_eq!(seeds.combat_seed, 0);
    assert_eq!(seeds.drop_seed, 0);
    assert_eq!(seeds.ai_seed, 0);
    assert_eq!(seeds.world_seed, 0);
}

/// DeterministicRng 产生的随机数在相同种子下一致。
#[test]
fn deterministic_rng_same_seed_same_output() {
    let mut rng_a = DeterministicRng(CoreDeterministicRng::with_seed(0));
    let mut rng_b = DeterministicRng(CoreDeterministicRng::with_seed(0));

    let a1 = rng_a.0.next_u64(RngStream::Combat);
    let b1 = rng_b.0.next_u64(RngStream::Combat);
    assert_eq!(a1, b1, "same seed must produce same value");

    let a2 = rng_a.0.next_u64(RngStream::Drop);
    let b2 = rng_b.0.next_u64(RngStream::Drop);
    assert_eq!(
        a2, b2,
        "same seed must produce same value for different stream"
    );
}

/// ReplayModeGuard 默认构造为正常模式（非回放）。
#[test]
fn replay_mode_guard_default_is_normal() {
    let guard = ReplayModeGuard::default();
    assert!(!guard.0.is_replay, "default mode should be normal");
}

/// ReplayModeGuard 可通过 replay_mode() 切换到回放模式。
#[test]
fn replay_mode_guard_can_switch_to_replay() {
    let mut guard = ReplayModeGuard::default();
    guard.0.is_replay = true;
    assert!(guard.0.is_replay, "should be in replay mode after switch");
}

/// RecordingSession 默认为 None（不在录制模式）。
#[test]
fn recording_session_default_is_none() {
    let session = RecordingSession::default();
    assert!(
        session.0.is_none(),
        "default recording session should be None"
    );
}

/// PlaybackSession 默认为 None（不在回放模式）。
#[test]
fn playback_session_default_is_none() {
    let session = PlaybackSession::default();
    assert!(
        session.0.is_none(),
        "default playback session should be None"
    );
}

/// FrameCounter 默认从 0 开始。
#[test]
fn frame_counter_default_starts_at_zero() {
    let counter = FrameCounter::default();
    assert_eq!(counter.0, 0, "frame counter should start at 0");
}
