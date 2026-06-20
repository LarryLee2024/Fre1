use crate::core::capabilities::runtime::scheduler::foundation::{
    SchedulerError, SchedulerState, TickPhase, TimeScale,
};

#[test]
fn scheduler_initial_state() {
    let state = SchedulerState::new();
    assert!(!state.initialized);
    assert!(!state.paused);
    assert!(state.current_time.is_initial());
}

#[test]
fn scheduler_initialization_succeeds() {
    let mut state = SchedulerState::new();
    assert!(state.initialize().is_ok());
    assert!(state.initialized);
    assert_eq!(state.current_time.phase, TickPhase::PreTick);
}

#[test]
fn scheduler_pause_resume() {
    let mut state = SchedulerState::new();
    state.pause();
    assert!(state.paused);
    state.resume();
    assert!(!state.paused);
}

#[test]
fn advance_frame() {
    let mut state = SchedulerState::new();
    state.initialize().unwrap();
    assert!(state.advance_frame().is_ok());
    assert_eq!(state.current_time.frame, 1);
}

#[test]
fn advance_frame_uninitialized_returns_error() {
    let mut state = SchedulerState::new();
    assert_eq!(state.advance_frame(), Err(SchedulerError::NotInitialized));
}

#[test]
fn advance_frame_paused_returns_error() {
    let mut state = SchedulerState::new();
    state.initialize().unwrap();
    state.pause();
    assert_eq!(state.advance_frame(), Err(SchedulerError::Paused));
}

#[test]
fn advance_phase() {
    let mut state = SchedulerState::new();
    state.initialize().unwrap(); // goes to PreTick
    state.advance_frame().unwrap(); // frame 1
    state.advance_phase().unwrap(); // goes to Tick, frame=0
    assert_eq!(state.current_time.phase, TickPhase::Tick);
    assert_eq!(state.current_time.frame, 0);
}

#[test]
fn advance_turn() {
    let mut state = SchedulerState::new();
    state.initialize().unwrap();
    state.advance_turn().unwrap();
    assert_eq!(state.current_time.turn, 1);
    assert_eq!(state.current_time.phase, TickPhase::PreTick);
}

#[test]
fn frame_overflow_limited() {
    let mut state = SchedulerState::new();
    state.initialize().unwrap();
    state.set_max_frames_per_phase(5);
    for _ in 0..5 {
        assert!(state.advance_frame().is_ok());
    }
    assert_eq!(state.advance_frame(), Err(SchedulerError::FrameOverflow { frame: 6 }));
}

#[test]
fn time_scale_default_value() {
    let scale = TimeScale::default();
    assert_eq!(scale.multiplier, 1.0);
}

#[test]
fn time_scale_clamping() {
    let scale = TimeScale::new(0.05); // below minimum
    assert!((scale.multiplier - 0.1).abs() < f32::EPSILON);

    let scale = TimeScale::new(20.0); // above maximum
    assert!((scale.multiplier - 10.0).abs() < f32::EPSILON);
}

#[test]
fn time_scale_frame_delay_calculation() {
    let scale = TimeScale::new(2.0);
    assert_eq!(scale.frame_delay(100), 50);

    let scale = TimeScale::new(0.5);
    assert_eq!(scale.frame_delay(100), 200);
}

#[test]
fn reinitialization_idempotent() {
    let mut state = SchedulerState::new();
    assert!(state.initialize().is_ok());
    assert!(state.initialize().is_ok()); // second call is no-op
}

#[test]
fn uninitialized_advance_phase_returns_error() {
    let mut state = SchedulerState::new();
    assert_eq!(state.advance_phase(), Err(SchedulerError::NotInitialized));
}

#[test]
fn uninitialized_advance_turn_returns_error() {
    let mut state = SchedulerState::new();
    assert_eq!(state.advance_turn(), Err(SchedulerError::NotInitialized));
}
