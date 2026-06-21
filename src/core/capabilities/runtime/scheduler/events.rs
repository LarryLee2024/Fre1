//! Scheduler 领域事件

use bevy::prelude::*;

use crate::core::capabilities::runtime::scheduler::foundation::{GameTime, TickPhase};

/// 新帧开始时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct FrameStarted {
    /// 帧开始的游戏时间
    pub game_time: GameTime,
    /// 自调度启动以来的帧序号
    pub frame_number: u64,
}

/// 阶段发生变化时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PhaseChanged {
    /// 变化时的游戏时间
    pub game_time: GameTime,
    /// 上一个阶段
    pub from: TickPhase,
    /// 当前阶段
    pub to: TickPhase,
}

/// 新回合开始时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct TurnStarted {
    /// 回合开始的游戏时间
    pub game_time: GameTime,
    /// 新回合序号
    pub turn_number: u32,
}

/// 调度被暂停时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SchedulingPaused {
    /// 暂停时的游戏时间
    pub game_time: GameTime,
}

/// 调度恢复时触发。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct SchedulingResumed {
    /// 恢复时的游戏时间
    pub game_time: GameTime,
}
