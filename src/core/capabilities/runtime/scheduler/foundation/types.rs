//! Scheduler 基础类型与枚举
//!
//! 定义游戏时间、帧阶段以及调度器领域错误。
//!
//! 详见 docs/01-architecture/20-tactical-combat/ADR-021-turn-state-machine.md

/// 游戏内时间（确定性时间表示，不依赖 wall-clock）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameTime {
    /// 已进行的回合数
    pub turn: u32,
    /// 当前帧阶段
    pub phase: TickPhase,
    /// 自阶段开始以来的帧计数器
    pub frame: u64,
}

impl GameTime {
    /// 初始游戏时间（turn=0, phase=Idle, frame=0）。
    pub const fn initial() -> Self {
        Self {
            turn: 0,
            phase: TickPhase::Idle,
            frame: 0,
        }
    }

    /// 是否为初始状态。
    pub fn is_initial(&self) -> bool {
        self.turn == 0 && self.phase == TickPhase::Idle && self.frame == 0
    }

    /// 推进到下一帧（同一阶段内帧计数增加）。
    pub fn advance_frame(mut self) -> Self {
        self.frame += 1;
        self
    }

    /// 推进到下一阶段（帧计数器重置，阶段改变）。
    pub fn advance_phase(mut self) -> Self {
        self.phase = self.phase.next();
        self.frame = 0;
        self
    }

    /// 推进到下一回合（阶段回到 PreTick，帧归零，回合+1）。
    pub fn advance_turn(mut self) -> Self {
        self.turn += 1;
        self.phase = TickPhase::PreTick;
        self.frame = 0;
        self
    }
}

/// 帧阶段——每一帧被细分为多个执行阶段。
///
/// 类似 Bevy 的 Schedule 标签逻辑，但面向游戏循环的确定性控制。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TickPhase {
    /// 帧前准备（输入收集、命令入队）
    PreTick,
    /// 帧核心更新（业务逻辑、管线执行）
    Tick,
    /// 帧后处理（事件响应、表现更新）
    PostTick,
    /// 空闲（暂停或帧间等待）
    Idle,
}

impl TickPhase {
    /// 返回下一个阶段。
    pub fn next(&self) -> Self {
        match self {
            Self::PreTick => Self::Tick,
            Self::Tick => Self::PostTick,
            Self::PostTick => Self::Idle,
            Self::Idle => Self::PreTick,
        }
    }

    /// 返回阶段名称。
    pub fn name(&self) -> &str {
        match self {
            Self::PreTick => "PreTick",
            Self::Tick => "Tick",
            Self::PostTick => "PostTick",
            Self::Idle => "Idle",
        }
    }
}

/// Scheduler 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerError {
    /// 调度器未初始化
    NotInitialized,
    /// 调度器已暂停
    Paused,
    /// 无效的阶段转换
    InvalidTransition { from: TickPhase, to: TickPhase },
    /// 帧计数器溢出
    FrameOverflow(u64),
}

impl std::fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "scheduler not initialized"),
            Self::Paused => write!(f, "scheduler is paused"),
            Self::InvalidTransition { from, to } => {
                write!(f, "invalid phase transition: {:?} → {:?}", from, to)
            }
            Self::FrameOverflow(count) => write!(f, "frame counter overflow at {}", count),
        }
    }
}

impl std::error::Error for SchedulerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_001_game_time_initial() {
        let t = GameTime::initial();
        assert_eq!(t.turn, 0);
        assert_eq!(t.phase, TickPhase::Idle);
        assert_eq!(t.frame, 0);
        assert!(t.is_initial());
    }

    #[test]
    fn unit_002_game_time_advance_frame() {
        let t = GameTime::initial().advance_frame();
        assert_eq!(t.frame, 1);
        assert!(!t.is_initial());
    }

    #[test]
    fn unit_003_game_time_advance_phase() {
        let t = GameTime::initial().advance_phase();
        assert_eq!(t.phase, TickPhase::PreTick);
        assert_eq!(t.frame, 0);
    }

    #[test]
    fn unit_004_game_time_advance_turn() {
        let t = GameTime::initial().advance_turn();
        assert_eq!(t.turn, 1);
        assert_eq!(t.phase, TickPhase::PreTick);
        assert_eq!(t.frame, 0);
    }

    #[test]
    fn unit_005_tick_phase_next() {
        assert_eq!(TickPhase::PreTick.next(), TickPhase::Tick);
        assert_eq!(TickPhase::Tick.next(), TickPhase::PostTick);
        assert_eq!(TickPhase::PostTick.next(), TickPhase::Idle);
        assert_eq!(TickPhase::Idle.next(), TickPhase::PreTick);
    }

    #[test]
    fn unit_006_tick_phase_name() {
        assert_eq!(TickPhase::PreTick.name(), "PreTick");
        assert_eq!(TickPhase::Tick.name(), "Tick");
        assert_eq!(TickPhase::PostTick.name(), "PostTick");
        assert_eq!(TickPhase::Idle.name(), "Idle");
    }

    #[test]
    fn unit_007_error_display() {
        let err = SchedulerError::NotInitialized;
        let msg = format!("{}", err);
        assert!(msg.contains("not initialized"));
    }

    #[test]
    fn unit_008_error_invalid_transition() {
        let err = SchedulerError::InvalidTransition {
            from: TickPhase::Idle,
            to: TickPhase::Tick,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Idle"));
    }
}
