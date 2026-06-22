use crate::core::capabilities::runtime::scheduler::foundation::{
    GameTime, SchedulerError, TickPhase,
};

#[test]
fn game_time_initial_state_correct() {
    let t = GameTime::initial();
    assert_eq!(t.turn, 0);
    assert_eq!(t.phase, TickPhase::Idle);
    assert_eq!(t.frame, 0);
    assert!(t.is_initial());
}

#[test]
fn game_time_advance_frame() {
    let t = GameTime::initial().advance_frame();
    assert_eq!(t.frame, 1);
    assert!(!t.is_initial());
}

#[test]
fn game_time_advance_phase() {
    let t = GameTime::initial().advance_phase();
    assert_eq!(t.phase, TickPhase::PreTick);
    assert_eq!(t.frame, 0);
}

#[test]
fn game_time_advance_turn() {
    let t = GameTime::initial().advance_turn();
    assert_eq!(t.turn, 1);
    assert_eq!(t.phase, TickPhase::PreTick);
    assert_eq!(t.frame, 0);
}

#[test]
fn tick_phase_next_phase_correct() {
    assert_eq!(TickPhase::PreTick.next(), TickPhase::Tick);
    assert_eq!(TickPhase::Tick.next(), TickPhase::PostTick);
    assert_eq!(TickPhase::PostTick.next(), TickPhase::Idle);
    assert_eq!(TickPhase::Idle.next(), TickPhase::PreTick);
}

#[test]
fn tick_phase_name_correct() {
    assert_eq!(TickPhase::PreTick.name(), "PreTick");
    assert_eq!(TickPhase::Tick.name(), "Tick");
    assert_eq!(TickPhase::PostTick.name(), "PostTick");
    assert_eq!(TickPhase::Idle.name(), "Idle");
}

#[test]
fn scheduler_error_display_correct() {
    let err = SchedulerError::NotInitialized;
    let msg = format!("{}", err);
    assert!(msg.contains("未初始化"));
}

#[test]
fn scheduler_error_invalid_transition_display() {
    let err = SchedulerError::InvalidTransition {
        from: TickPhase::Idle,
        to: TickPhase::Tick,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Idle"));
}
