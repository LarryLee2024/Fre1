//! Scheduler 值对象：调度器运行时状态

use super::types::{GameTime, SchedulerError, TickPhase};

/// 调度器运行时状态。
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerState {
    /// 当前游戏时间
    pub current_time: GameTime,
    /// 是否已初始化
    pub initialized: bool,
    /// 是否暂停
    pub paused: bool,
    /// 每 Tick 阶段的帧数上限（0 = 无限制）
    pub max_frames_per_phase: u64,
    /// 每回合的帧数上限（0 = 无限制）
    pub max_frames_per_turn: u64,
}

impl SchedulerState {
    /// 创建初始调度器状态。
    pub fn new() -> Self {
        Self {
            current_time: GameTime::initial(),
            initialized: false,
            paused: false,
            max_frames_per_phase: 0,
            max_frames_per_turn: 0,
        }
    }

    /// 初始化调度器（从 Idle 进入 PreTick）。
    pub fn initialize(&mut self) -> Result<(), SchedulerError> {
        if self.initialized {
            return Ok(());
        }
        self.current_time = GameTime::initial().advance_phase(); // Idle → PreTick
        self.initialized = true;
        Ok(())
    }

    /// 暂停调度。
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// 恢复调度。
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// 推进一帧（同一阶段内帧计数增加）。
    pub fn advance_frame(&mut self) -> Result<(), SchedulerError> {
        if !self.initialized {
            return Err(SchedulerError::NotInitialized);
        }
        if self.paused {
            return Err(SchedulerError::Paused);
        }

        let next_frame = self.current_time.frame + 1;

        // 检查阶段帧数上限
        if self.max_frames_per_phase > 0 && next_frame > self.max_frames_per_phase {
            return Err(SchedulerError::FrameOverflow(next_frame));
        }

        // 检查回合帧数上限
        if self.max_frames_per_turn > 0 && self.current_time.turn > 0 {
            let total_frames_this_turn = next_frame;
            if total_frames_this_turn > self.max_frames_per_turn {
                return Err(SchedulerError::FrameOverflow(next_frame));
            }
        }

        self.current_time.frame = next_frame;
        Ok(())
    }

    /// 推进到下一阶段。
    pub fn advance_phase(&mut self) -> Result<(), SchedulerError> {
        if !self.initialized {
            return Err(SchedulerError::NotInitialized);
        }

        let next_phase = self.current_time.phase.next();
        self.current_time.phase = next_phase;
        self.current_time.frame = 0;
        Ok(())
    }

    /// 推进到下一回合。
    pub fn advance_turn(&mut self) -> Result<(), SchedulerError> {
        if !self.initialized {
            return Err(SchedulerError::NotInitialized);
        }

        let next_turn = self.current_time.turn + 1;
        self.current_time.turn = next_turn;
        self.current_time.phase = TickPhase::PreTick;
        self.current_time.frame = 0;
        Ok(())
    }

    /// 设置每阶段的帧数上限。
    pub fn set_max_frames_per_phase(&mut self, max: u64) {
        self.max_frames_per_phase = max;
    }

    /// 设置每回合的帧数上限。
    pub fn set_max_frames_per_turn(&mut self, max: u64) {
        self.max_frames_per_turn = max;
    }
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

/// 时间缩放因子——影响游戏速度。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScale {
    /// 缩放倍率（1.0 = 正常速度，2.0 = 两倍速，0.5 = 半速）
    pub multiplier: f32,
}

impl TimeScale {
    /// 创建时间缩放因子。
    pub fn new(multiplier: f32) -> Self {
        Self {
            multiplier: multiplier.max(0.1).min(10.0),
        }
    }

    /// 返回实际帧延迟（基于基准延迟）。
    pub fn frame_delay(&self, base_delay_ms: u64) -> u64 {
        (base_delay_ms as f64 / self.multiplier as f64) as u64
    }
}

impl Default for TimeScale {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_020_scheduler_initial_state() {
        let state = SchedulerState::new();
        assert!(!state.initialized);
        assert!(!state.paused);
        assert!(state.current_time.is_initial());
    }

    #[test]
    fn unit_021_scheduler_initialize() {
        let mut state = SchedulerState::new();
        assert!(state.initialize().is_ok());
        assert!(state.initialized);
        assert_eq!(state.current_time.phase, TickPhase::PreTick);
    }

    #[test]
    fn unit_022_scheduler_pause_resume() {
        let mut state = SchedulerState::new();
        state.pause();
        assert!(state.paused);
        state.resume();
        assert!(!state.paused);
    }

    #[test]
    fn unit_023_advance_frame() {
        let mut state = SchedulerState::new();
        state.initialize().unwrap();
        assert!(state.advance_frame().is_ok());
        assert_eq!(state.current_time.frame, 1);
    }

    #[test]
    fn unit_024_advance_frame_not_initialized() {
        let mut state = SchedulerState::new();
        assert_eq!(state.advance_frame(), Err(SchedulerError::NotInitialized));
    }

    #[test]
    fn unit_025_advance_frame_paused() {
        let mut state = SchedulerState::new();
        state.initialize().unwrap();
        state.pause();
        assert_eq!(state.advance_frame(), Err(SchedulerError::Paused));
    }

    #[test]
    fn unit_026_advance_phase() {
        let mut state = SchedulerState::new();
        state.initialize().unwrap(); // goes to PreTick
        state.advance_frame().unwrap(); // frame 1
        state.advance_phase().unwrap(); // goes to Tick, frame=0
        assert_eq!(state.current_time.phase, TickPhase::Tick);
        assert_eq!(state.current_time.frame, 0);
    }

    #[test]
    fn unit_027_advance_turn() {
        let mut state = SchedulerState::new();
        state.initialize().unwrap();
        state.advance_turn().unwrap();
        assert_eq!(state.current_time.turn, 1);
        assert_eq!(state.current_time.phase, TickPhase::PreTick);
    }

    #[test]
    fn unit_028_frame_overflow_limit() {
        let mut state = SchedulerState::new();
        state.initialize().unwrap();
        state.set_max_frames_per_phase(5);
        for _ in 0..5 {
            assert!(state.advance_frame().is_ok());
        }
        assert_eq!(state.advance_frame(), Err(SchedulerError::FrameOverflow(6)));
    }

    #[test]
    fn unit_029_time_scale_default() {
        let scale = TimeScale::default();
        assert_eq!(scale.multiplier, 1.0);
    }

    #[test]
    fn unit_030_time_scale_clamp() {
        let scale = TimeScale::new(0.05); // below minimum
        assert!((scale.multiplier - 0.1).abs() < f32::EPSILON);

        let scale = TimeScale::new(20.0); // above maximum
        assert!((scale.multiplier - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn unit_031_time_scale_frame_delay() {
        let scale = TimeScale::new(2.0);
        assert_eq!(scale.frame_delay(100), 50);

        let scale = TimeScale::new(0.5);
        assert_eq!(scale.frame_delay(100), 200);
    }

    #[test]
    fn unit_032_double_initialize() {
        let mut state = SchedulerState::new();
        assert!(state.initialize().is_ok());
        assert!(state.initialize().is_ok()); // second call is no-op
    }

    #[test]
    fn unit_033_advance_phase_not_initialized() {
        let mut state = SchedulerState::new();
        assert_eq!(state.advance_phase(), Err(SchedulerError::NotInitialized));
    }

    #[test]
    fn unit_034_advance_turn_not_initialized() {
        let mut state = SchedulerState::new();
        assert_eq!(state.advance_turn(), Err(SchedulerError::NotInitialized));
    }
}
