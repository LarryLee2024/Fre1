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
