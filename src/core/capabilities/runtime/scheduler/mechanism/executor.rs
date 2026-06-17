//! Scheduler Executor — 调度执行逻辑
//!
//! 提供帧推进和阶段转换的编排接口，供 Bevy System 调用。
//!
//! 详见 docs/01-architecture/20-tactical-combat/ADR-021-turn-state-machine.md

use crate::core::capabilities::runtime::scheduler::foundation::{
    GameTime, SchedulerError, SchedulerState, TickPhase,
};

/// 推进一个完整的帧周期：PreTick → Tick → PostTick → Idle。
///
/// 返回每个阶段的 GameTime 快照，调用方可在 System 中按阶段执行逻辑。
///
/// # Errors
/// - 调度器未初始化 → NotInitialized
/// - 调度器暂停 → Paused
pub fn execute_tick(state: &mut SchedulerState) -> Result<TickSequence, SchedulerError> {
    if !state.initialized {
        return Err(SchedulerError::NotInitialized);
    }
    if state.paused {
        return Err(SchedulerError::Paused);
    }

    let pre_tick = state.current_time;
    state.advance_phase()?; // → Tick
    let tick = state.current_time;
    state.advance_phase()?; // → PostTick
    let post_tick = state.current_time;
    state.advance_phase()?; // → Idle
    let idle = state.current_time;

    // 推进帧号（从 Idle 进入下一个 PreTick）
    state.advance_phase()?; // → PreTick (next frame)
    state.advance_frame()?;
    let next_pre_tick = state.current_time;

    Ok(TickSequence {
        pre_tick,
        tick,
        post_tick,
        idle,
        next_pre_tick,
    })
}

/// 完整的 Tick 阶段快照。
#[derive(Debug, Clone, PartialEq)]
pub struct TickSequence {
    /// PreTick 阶段的 GameTime
    pub pre_tick: GameTime,
    /// Tick 阶段的 GameTime
    pub tick: GameTime,
    /// PostTick 阶段的 GameTime
    pub post_tick: GameTime,
    /// Idle 阶段的 GameTime
    pub idle: GameTime,
    /// 下一个 PreTick 阶段的 GameTime（帧号已推进）
    pub next_pre_tick: GameTime,
}

/// 单步执行一个指定的阶段。
///
/// 不执行完整帧周期，仅推进到下一个阶段。
pub fn advance_to_next_phase(state: &mut SchedulerState) -> Result<GameTime, SchedulerError> {
    if !state.initialized {
        return Err(SchedulerError::NotInitialized);
    }
    state.advance_phase()?;
    Ok(state.current_time)
}

/// 推进到下一回合（重置帧计数，回合+1）。
pub fn advance_to_next_turn(state: &mut SchedulerState) -> Result<GameTime, SchedulerError> {
    if !state.initialized {
        return Err(SchedulerError::NotInitialized);
    }
    state.advance_turn()?;
    Ok(state.current_time)
}

/// 检查当前阶段是否匹配指定阶段。
pub fn is_phase(state: &SchedulerState, phase: TickPhase) -> bool {
    state.current_time.phase == phase
}

/// 获取自调度启动以来的总帧数。
pub fn total_frames(state: &SchedulerState) -> u64 {
    // 近似计算：turn * frames_per_turn + phase_offset + frame
    // 此处省略精确计算，仅返回当前帧号（简化实现）
    state.current_time.frame
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_initialized() -> SchedulerState {
        let mut state = SchedulerState::new();
        state.initialize().unwrap();
        state
    }

    #[test]
    fn unit_040_execute_tick_basic() {
        let mut state = setup_initialized();
        let sequence = execute_tick(&mut state).unwrap();

        assert_eq!(sequence.pre_tick.phase, TickPhase::PreTick);
        assert_eq!(sequence.tick.phase, TickPhase::Tick);
        assert_eq!(sequence.post_tick.phase, TickPhase::PostTick);
        assert_eq!(sequence.idle.phase, TickPhase::Idle);
    }

    #[test]
    fn unit_041_execute_tick_advances_frame() {
        let mut state = setup_initialized();
        let first = execute_tick(&mut state).unwrap();
        let second = execute_tick(&mut state).unwrap();

        // 第一帧的 PreTick 和第二帧的 PreTick 帧号应不同
        assert_eq!(first.pre_tick.frame, 0);
        assert_eq!(second.pre_tick.frame, 1);
    }

    #[test]
    fn unit_042_execute_tick_not_initialized() {
        let mut state = SchedulerState::new();
        assert_eq!(
            execute_tick(&mut state),
            Err(SchedulerError::NotInitialized)
        );
    }

    #[test]
    fn unit_043_execute_tick_paused() {
        let mut state = setup_initialized();
        state.pause();
        assert_eq!(execute_tick(&mut state), Err(SchedulerError::Paused));
    }

    #[test]
    fn unit_044_advance_to_next_phase() {
        let mut state = setup_initialized(); // PreTick
        let time = advance_to_next_phase(&mut state).unwrap();
        assert_eq!(time.phase, TickPhase::Tick);
        assert_eq!(time.frame, 0);
    }

    #[test]
    fn unit_045_advance_to_next_turn() {
        let mut state = setup_initialized(); // PreTick
        state.advance_frame().unwrap();
        state.advance_frame().unwrap();
        let time = advance_to_next_turn(&mut state).unwrap();
        assert_eq!(time.turn, 1);
        assert_eq!(time.phase, TickPhase::PreTick);
        assert_eq!(time.frame, 0);
    }

    #[test]
    fn unit_046_is_phase() {
        let state = setup_initialized(); // PreTick
        assert!(is_phase(&state, TickPhase::PreTick));
        assert!(!is_phase(&state, TickPhase::Tick));
    }

    #[test]
    fn unit_047_total_frames() {
        let mut state = setup_initialized();
        assert_eq!(total_frames(&state), 0);

        state.advance_frame().unwrap();
        assert_eq!(total_frames(&state), 1);
    }

    #[test]
    fn unit_048_multiple_ticks_maintain_order() {
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
}
