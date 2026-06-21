//! Scheduler 值对象：调度器运行时状态

use super::error::SchedulerError;
use super::types::{GameTime, TickPhase};

/// 调度器运行时状态。
///
/// 管理游戏时间推进、暂停/恢复和帧数上限。
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
    /// 创建初始调度器状态，未初始化且未暂停。
    pub fn new() -> Self {
        Self {
            current_time: GameTime::initial(),
            initialized: false,
            paused: false,
            max_frames_per_phase: 0,
            max_frames_per_turn: 0,
        }
    }

    /// 初始化调度器，从 Idle 进入 PreTick。
    ///
    /// 幂等：已初始化时直接返回 Ok。
    pub fn initialize(&mut self) -> Result<(), SchedulerError> {
        if self.initialized {
            return Ok(());
        }
        self.current_time = GameTime::initial().advance_phase(); // Idle → PreTick
        self.initialized = true;
        Ok(())
    }

    /// 暂停调度，暂停期间 advance_frame 和 advance_phase 返回 Paused 错误。
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// 恢复调度。
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// 推进一帧（同一阶段内帧计数增加）。
    ///
    /// # Errors
    /// - 未初始化 → NotInitialized
    /// - 已暂停 → Paused
    /// - 超过阶段或回合帧数上限 → FrameOverflow
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
            return Err(SchedulerError::FrameOverflow { frame: next_frame });
        }

        // 检查回合帧数上限
        if self.max_frames_per_turn > 0 && self.current_time.turn > 0 {
            let total_frames_this_turn = next_frame;
            if total_frames_this_turn > self.max_frames_per_turn {
                return Err(SchedulerError::FrameOverflow { frame: next_frame });
            }
        }

        self.current_time.frame = next_frame;
        Ok(())
    }

    /// 推进到下一阶段，帧计数器重置为 0。
    ///
    /// # Errors
    /// - 未初始化 → NotInitialized
    pub fn advance_phase(&mut self) -> Result<(), SchedulerError> {
        if !self.initialized {
            return Err(SchedulerError::NotInitialized);
        }

        let next_phase = self.current_time.phase.next();
        self.current_time.phase = next_phase;
        self.current_time.frame = 0;
        Ok(())
    }

    /// 推进到下一回合，阶段回到 PreTick，帧归零，回合 +1。
    ///
    /// # Errors
    /// - 未初始化 → NotInitialized
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

    /// 设置每阶段的最大帧数上限，超过时 advance_frame 返回 FrameOverflow。
    pub fn set_max_frames_per_phase(&mut self, max: u64) {
        self.max_frames_per_phase = max;
    }

    /// 设置每回合的最大帧数上限，超过时 advance_frame 返回 FrameOverflow。
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
///
/// multiplier > 1.0 加速，< 1.0 减速。clamp 到 [0.1, 10.0] 范围。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeScale {
    /// 缩放倍率（1.0 = 正常速度，2.0 = 两倍速，0.5 = 半速）
    pub multiplier: f32,
}

impl TimeScale {
    /// 创建时间缩放因子，multiplier 自动 clamp 到 [0.1, 10.0]。
    pub fn new(multiplier: f32) -> Self {
        Self {
            multiplier: multiplier.clamp(0.1, 10.0),
        }
    }

    /// 根据基准延迟计算实际帧延迟。
    ///
    /// multiplier=2.0 时帧延迟减半（两倍速），multiplier=0.5 时帧延迟翻倍（半速）。
    pub fn frame_delay(&self, base_delay_ms: u64) -> u64 {
        (base_delay_ms as f64 / self.multiplier as f64) as u64
    }
}

impl Default for TimeScale {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}
