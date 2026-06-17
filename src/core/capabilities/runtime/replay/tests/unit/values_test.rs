use crate::core::capabilities::runtime::replay::foundation::{
    DeterministicRng, ReplayCommand, ReplayFrame, ReplayHeader, ReplayLog, ReplayMode,
    ReplayModeGuard, ReplayPlayer, ReplayRecorder, ReplayValidator, RngSeeds, RngStream,
};

// ── DeterministicRng ─────────────────────────────

#[test]
fn verify_initial_rng_uniform_seed() {
    let mut rng = DeterministicRng::with_seed(42);
    let val1 = rng.next_u64(RngStream::Combat);
    let val2 = rng.next_u64(RngStream::Combat);
    // Same seed, same first call → should be deterministic
    let mut rng2 = DeterministicRng::with_seed(42);
    assert_eq!(val1, rng2.next_u64(RngStream::Combat));
    // Second call should differ from first
    assert_ne!(val1, val2);
}

#[test]
fn verify_different_rng_different_stream() {
    let mut rng = DeterministicRng::with_seed(100);
    let combat = rng.next_u64(RngStream::Combat);
    let drop = rng.next_u64(RngStream::Drop);
    // Different streams with same seed → different values due to stream offset
    assert_ne!(combat, drop);
}

#[test]
fn verify_rng_f32_range() {
    let mut rng = DeterministicRng::with_seed(42);
    for _ in 0..100 {
        let val = rng.next_f32(RngStream::Combat);
        assert!(val >= 0.0);
        assert!(val < 1.0);
    }
}

#[test]
fn verify_rng_bool_probability() {
    let mut rng = DeterministicRng::with_seed(42);
    let count_true = (0..1000)
        .filter(|_| rng.gen_bool(RngStream::Combat, 0.5))
        .count();
    // With 50% probability, roughly half should be true
    assert!(count_true > 300 && count_true < 700);
}

#[test]
fn verify_rng_integer_range() {
    let mut rng = DeterministicRng::with_seed(42);
    for _ in 0..100 {
        let val = rng.gen_range(RngStream::Combat, 5, 10);
        assert!(val >= 5);
        assert!(val < 10);
    }
}

#[test]
fn verify_rng_restore_seed() {
    let mut rng = DeterministicRng::with_seed(42);
    let first = rng.next_u64(RngStream::Combat);

    rng.set_seed(RngStream::Combat, 42);
    let after_reset = rng.next_u64(RngStream::Combat);
    assert_eq!(first, after_reset);
}

#[test]
fn verify_rng_batch_set_seeds() {
    let mut rng = DeterministicRng::with_seed(1);
    let seeds = RngSeeds::new(10, 20, 30, 40);
    rng.set_all_seeds(seeds);
    assert_eq!(rng.get_seed(RngStream::Combat), 10);
    assert_eq!(rng.get_seed(RngStream::Drop), 20);
}

// ── ReplayRecorder ───────────────────────────────

#[test]
fn recorder_initial_state() {
    let recorder = ReplayRecorder::new(60);
    assert!(!recorder.is_recording);
    assert_eq!(recorder.frame_count(), 0);
}

#[test]
fn recorder_start_stop() {
    let mut recorder = ReplayRecorder::new(60);
    recorder.start_recording(0);
    assert!(recorder.is_recording);

    recorder.stop_recording();
    assert!(!recorder.is_recording);
}

#[test]
fn recorder_record_command() {
    let mut recorder = ReplayRecorder::new(60);
    recorder.start_recording(0);

    recorder.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
    assert_eq!(recorder.commands_in_current_frame(), 1);
}

#[test]
fn recorder_frame_boundary() {
    let mut recorder = ReplayRecorder::new(60);
    recorder.start_recording(100);

    recorder.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
    recorder.start_frame(1, 101);
    recorder.record_command(ReplayCommand::SkipTurn { unit: "u2".into() });

    recorder.stop_recording();
    assert_eq!(recorder.frame_count(), 2);
}

#[test]
fn recorder_checkpoint_frame_detection() {
    let recorder = ReplayRecorder::new(10);
    assert!(recorder.is_checkpoint_frame(0));
    assert!(!recorder.is_checkpoint_frame(1));
    assert!(recorder.is_checkpoint_frame(10));
}

#[test]
fn command_ignored_when_not_recording() {
    let mut recorder = ReplayRecorder::new(60);
    recorder.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
    // Not recording → commands should be ignored
    assert_eq!(recorder.commands_in_current_frame(), 0);
}

// ── ReplayPlayer ─────────────────────────────────

#[test]
fn player_initial_state() {
    let player = ReplayPlayer::new(ReplayMode::Full);
    assert!(!player.is_playing);
    assert!(player.is_finished());
}

#[test]
fn player_load_and_play() {
    let log = create_test_replay_log();
    let mut player = ReplayPlayer::new(ReplayMode::Full);

    player.load(&log);
    player.start_playing();
    assert!(player.is_playing);
    assert!(!player.is_finished());
}

#[test]
fn player_advance_frame() {
    let log = create_test_replay_log();
    let mut player = ReplayPlayer::new(ReplayMode::Full);

    player.load(&log);
    player.start_playing();
    assert!(player.advance_frame());
    assert_eq!(player.current_frame_number(), Some(1));
}

#[test]
fn player_current_commands() {
    let log = create_test_replay_log();
    let mut player = ReplayPlayer::new(ReplayMode::Full);

    player.load(&log);
    player.start_playing();

    let cmds = player.current_commands();
    assert!(!cmds.is_empty());
    assert_eq!(cmds[0].type_name(), "SkipTurn");
}

#[test]
fn player_playback_ends() {
    let log = create_test_replay_log();
    let mut player = ReplayPlayer::new(ReplayMode::Full);

    player.load(&log);
    player.start_playing();
    player.advance_frame();
    // After advancing past the last frame
    assert!(!player.advance_frame()); // no more frames
}

#[test]
fn replay_mode_name() {
    assert_eq!(ReplayMode::Full.name(), "Full");
    assert_eq!(ReplayMode::FastForward.name(), "FastForward");
    assert_eq!(ReplayMode::StepByStep.name(), "StepByStep");
}

// ── ReplayValidator ──────────────────────────────

#[test]
fn validator_initial_state() {
    let v = ReplayValidator::new();
    assert_eq!(v.current_frame, 0);
    assert!(!v.has_mismatches());
}

#[test]
fn verify_checksum_no_mismatch() {
    let mut v = ReplayValidator::new();
    v.start_verification();
    v.verify_checksum(0, 0xABCD, 0xABCD);
    assert!(!v.has_mismatches());
}

#[test]
fn verify_checksum_mismatch_detected() {
    let mut v = ReplayValidator::new();
    v.start_verification();
    v.verify_checksum(0, 0xABCD, 0xDEAD);
    assert!(v.has_mismatches());
    assert_eq!(v.mismatch_count(), 1);
}

#[test]
fn accumulated_checksum_correct() {
    let mut v = ReplayValidator::new();
    v.start_recording();
    v.record_checksum(0xAAAA);
    v.record_checksum(0xBBBB);
    // XOR of both
    assert_eq!(v.accumulated_checksum(), 0xAAAA ^ 0xBBBB);
}

#[test]
fn replay_mode_guard() {
    let guard = ReplayModeGuard::normal();
    assert!(!guard.is_replay);

    let guard = ReplayModeGuard::replay_mode();
    assert!(guard.is_replay);
}

// ── ReplayLog ────────────────────────────────────

#[test]
fn replay_log_construction() {
    let header = ReplayHeader::new(1, "1.0", "scene_001", 42);
    let log = ReplayLog::new(header);
    assert_eq!(log.frame_count(), 0);
}

#[test]
fn replay_log_add_frame() {
    let header = ReplayHeader::new(1, "1.0", "scene_001", 42);
    let mut log = ReplayLog::new(header);
    log.add_frame(ReplayFrame::new(0, 100));
    log.add_frame(ReplayFrame::new(1, 101));
    assert_eq!(log.frame_count(), 2);
}

#[test]
fn replay_log_final_checksum() {
    let header = ReplayHeader::new(1, "1.0", "scene", 0);
    let mut log = ReplayLog::new(header);
    log.set_final_checksum(0x1234);
    assert_eq!(log.final_checksum, Some(0x1234));
}

// ── Helpers ──────────────────────────────────────

fn create_test_replay_log() -> ReplayLog {
    let header = ReplayHeader::new(1, "1.0", "test_scene", 42);

    let mut frame0 = ReplayFrame::new(0, 100);
    frame0.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

    let mut frame1 = ReplayFrame::new(1, 101);
    frame1.add_command(ReplayCommand::UnitMove {
        unit: "u1".into(),
        path: vec!["0,0".into(), "1,0".into()],
    });

    let mut log = ReplayLog::new(header);
    log.add_frame(frame0);
    log.add_frame(frame1);
    log
}
