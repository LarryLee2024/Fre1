use crate::core::capabilities::runtime::replay::foundation::{
    ReplayCommand, ReplayError, ReplayFrame, ReplayHeader,
};
use crate::core::capabilities::runtime::replay::mechanism::recorder::*;

#[test]
fn recording_session_lifecycle() {
    let mut session = RecordingSession::new(60);
    let header = ReplayHeader::new(1, "1.0", "test_scene", 42);

    session.start(header, 100);
    assert!(session.is_recording());

    session.record_command(ReplayCommand::SkipTurn { unit: "u1".into() });
    session.start_frame(1, 101);
    session.record_command(ReplayCommand::UnitMove {
        unit: "u1".into(),
        path: vec!["0,0".into()],
    });
    session.finalize_frame(0xAAAA);

    let log = session.stop(0xBBBB).unwrap();
    assert_eq!(log.frame_count(), 2);
}

#[test]
fn stop_fails_when_not_recording() {
    let mut session = RecordingSession::new(60);
    let result = session.stop(0);
    assert_eq!(result, Err(ReplayError::NotRecording));
}

#[test]
fn frame_checksum_calculation() {
    let mut frame = ReplayFrame::new(0, 100);
    frame.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

    let checksum = calculate_frame_checksum(&frame);
    assert_ne!(checksum, 0);
}

#[test]
fn frame_checksum_deterministic() {
    let mut frame = ReplayFrame::new(0, 100);
    frame.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

    let c1 = calculate_frame_checksum(&frame);
    let c2 = calculate_frame_checksum(&frame);
    assert_eq!(c1, c2);
}

#[test]
fn frame_sequence_consecutive_passes() {
    let frames = vec![ReplayFrame::new(0, 0), ReplayFrame::new(1, 0)];
    assert!(validate_frame_sequence(&frames).is_ok());
}

#[test]
fn frame_sequence_gap_detected() {
    let frames = vec![ReplayFrame::new(0, 0), ReplayFrame::new(2, 0)];
    assert_eq!(
        validate_frame_sequence(&frames),
        Err(ReplayError::FrameNumberGap {
            expected: 1,
            got: 2
        })
    );
}

#[test]
fn version_validation_passes() {
    assert!(validate_version(1, 2).is_ok());
    assert!(validate_version(1, 1).is_ok());
}

#[test]
fn version_mismatch_detected() {
    assert_eq!(
        validate_version(2, 1),
        Err(ReplayError::VersionMismatch {
            expected: 1,
            actual: 2
        })
    );
}

#[test]
fn different_content_different_checksum() {
    let mut frame_a = ReplayFrame::new(0, 0);
    frame_a.add_command(ReplayCommand::SkipTurn { unit: "u1".into() });

    let mut frame_b = ReplayFrame::new(0, 0);
    frame_b.add_command(ReplayCommand::SkipTurn { unit: "u2".into() });

    assert_ne!(
        calculate_frame_checksum(&frame_a),
        calculate_frame_checksum(&frame_b)
    );
}
