use crate::core::capabilities::runtime::scheduler::foundation::{
    SchedulerError, SchedulerState, TickPhase,
};
use crate::core::capabilities::runtime::scheduler::mechanism::executor::*;

fn setup_initialized() -> SchedulerState {
    let mut state = SchedulerState::new();
    state.initialize().unwrap();
    state
}

#[test]
fn execute_basic_tick() {
    let mut state = setup_initialized();
    let sequence = execute_tick(&mut state).unwrap();

    assert_eq!(sequence.pre_tick.phase, TickPhase::PreTick);
    assert_eq!(sequence.tick.phase, TickPhase::Tick);
    assert_eq!(sequence.post_tick.phase, TickPhase::PostTick);
    assert_eq!(sequence.idle.phase, TickPhase::Idle);
}

#[test]
fn execute_tick_advances_frame() {
    let mut state = setup_initialized();
    let first = execute_tick(&mut state).unwrap();
    let second = execute_tick(&mut state).unwrap();

    // 第一帧的 PreTick 和第二帧的 PreTick 帧号应不同
    assert_eq!(first.pre_tick.frame, 0);
    assert_eq!(second.pre_tick.frame, 1);
}

#[test]
fn uninitialized_tick_returns_error() {
    let mut state = SchedulerState::new();
    assert_eq!(
        execute_tick(&mut state),
        Err(SchedulerError::NotInitialized)
    );
}

#[test]
fn paused_tick_returns_error() {
    let mut state = setup_initialized();
    state.pause();
    assert_eq!(execute_tick(&mut state), Err(SchedulerError::Paused));
}

#[test]
fn test_advance_to_next_phase() {
    let mut state = setup_initialized(); // PreTick
    let time = advance_to_next_phase(&mut state).unwrap();
    assert_eq!(time.phase, TickPhase::Tick);
    assert_eq!(time.frame, 0);
}

#[test]
fn test_advance_to_next_turn() {
    let mut state = setup_initialized(); // PreTick
    state.advance_frame().unwrap();
    state.advance_frame().unwrap();
    let time = advance_to_next_turn(&mut state).unwrap();
    assert_eq!(time.turn, 1);
    assert_eq!(time.phase, TickPhase::PreTick);
    assert_eq!(time.frame, 0);
}

#[test]
fn determine_current_phase() {
    let state = setup_initialized(); // PreTick
    assert!(is_phase(&state, TickPhase::PreTick));
    assert!(!is_phase(&state, TickPhase::Tick));
}

#[test]
fn get_total_frame_count() {
    let mut state = setup_initialized();
    assert_eq!(total_frames(&state), 0);

    state.advance_frame().unwrap();
    assert_eq!(total_frames(&state), 1);
}

#[test]
fn multiple_ticks_maintain_phase_order() {
    let mut state = setup_initialized();

    for i in 0..5 {
        let seq = execute_tick(&mut state).unwrap();

        // Phase 顺序校验——每个 TickSequence 应依次经过 PreTick → Tick → PostTick → Idle
        assert_eq!(seq.pre_tick.phase, TickPhase::PreTick);
        assert_eq!(seq.tick.phase, TickPhase::Tick);
        assert_eq!(seq.post_tick.phase, TickPhase::PostTick);
        assert_eq!(seq.idle.phase, TickPhase::Idle);
        assert_eq!(seq.next_pre_tick.phase, TickPhase::PreTick);

        // GameTime.frame 是"自阶段开始以来的帧计数器"（per-phase），
        // 每调用一次 execute_tick 后 frame=1（0 → advance_frame → 1）。
        // 使用 total_frames 确认状态已推进。
        assert_eq!(total_frames(&state), 1);
    }

    // 5 次 tick 后：状态位于 PreTick，frame=1
    assert_eq!(state.current_time.phase, TickPhase::PreTick);
    assert_eq!(state.current_time.frame, 1);
}
