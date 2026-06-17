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
