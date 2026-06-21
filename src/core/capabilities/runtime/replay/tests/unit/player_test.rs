use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayError, ReplayFrame, ReplayHeader, ReplayLog, ReplayMode,
};
use crate::core::capabilities::runtime::replay::mechanism::player::*;
use crate::shared::random::RngStream;

fn create_test_log() -> ReplayLog {
    let header = ReplayHeader::new(1, "1.0", "test", 42);

    let mut frame0 = ReplayFrame::new(0, 0);
    frame0.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

    let mut frame1 = ReplayFrame::new(1, 1);
    frame1.add_command(ReplayCommand::UnitMove {
        unit: "u1".into(),
        path: vec!["0,0".into()],
    });

    let mut log = ReplayLog::new(header);
    log.add_frame(frame0);
    log.add_frame(frame1);
    log
}

#[test]
fn replay_session_load_succeeds() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    assert!(session.load(&log).is_ok());
}

#[test]
fn empty_log_load_fails() {
    let header = ReplayHeader::new(1, "1.0", "test", 0);
    let log = ReplayLog::new(header);
    let mut session = PlaybackSession::new(ReplayMode::Full, 0);
    assert_eq!(session.load(&log), Err(ReplayError::EmptyLog));
}

#[test]
fn replay_starts_and_advances_frame() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).unwrap();
    session.start();

    assert_eq!(session.current_frame_number(), Some(0));
    assert!(session.advance_frame());
    assert_eq!(session.current_frame_number(), Some(1));
    assert!(!session.advance_frame()); // no more frames
    assert!(session.is_finished());
}

#[test]
fn get_current_frame_commands() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).unwrap();
    session.start();

    let cmds = session.current_commands();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].type_name(), "SkipTurn");
}

#[test]
fn rng_determinism_verified() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).unwrap();
    session.start();

    let val_a = session.rng_mut().next_u64(RngStream::Combat);

    // Create another session with same seed
    let log2 = create_test_log();
    let mut session2 = PlaybackSession::new(ReplayMode::Full, 42);
    session2.load(&log2).unwrap();
    session2.start();
    let val_b = session2.rng_mut().next_u64(RngStream::Combat);

    assert_eq!(val_a, val_b);
}

#[test]
fn verify_frame_checksum() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).unwrap();
    session.start();

    // Frame has no checksum set, so verification should pass
    assert!(session.verify_current_frame().is_ok());
}

#[test]
fn version_mismatch_rejected() {
    let header = ReplayHeader::new(2, "2.0", "test", 0);
    let mut frame = ReplayFrame::new(0, 0);
    frame.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });
    let mut log = ReplayLog::new(header);
    log.add_frame(frame);

    let mut session = PlaybackSession::new(ReplayMode::Full, 0);
    assert!(session.load(&log).is_err());
}

#[test]
fn fast_forward_completes() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::FastForward, 42);
    session.load(&log).unwrap();
    session.start();

    fast_forward(&mut session).unwrap();
    assert!(session.is_finished());
}

#[test]
fn replay_session_stops() {
    let log = create_test_log();
    let mut session = PlaybackSession::new(ReplayMode::Full, 42);
    session.load(&log).unwrap();
    session.start();
    assert!(session.player.is_playing);

    session.stop();
    assert!(!session.player.is_playing);
}
