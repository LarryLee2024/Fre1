//! 游戏内时间系统
//!
//! 基于帧计数的确定性时间系统，不依赖任何 wall-clock。
//! 确保回放和数据流确定性。
//!
//! 详见 `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md`
//!
//! # 核心类型
//! - [`GameTime`]: (frame, turn) 二元组，标记游戏中每一个时间点

use bevy::prelude::*;

/// 游戏内时间，基于帧计数和回合计数。
///
/// 确定性时间系统，不依赖系统时钟。
/// (frame, turn) 二元组标记游戏中的每一个时间点。
///
/// 作为 Resource 注册到 App，所有需要时间感知的系统通过 `Res<GameTime>` 访问。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Resource)]
pub struct GameTime {
    frame: u64,
    turn: u64,
}

impl GameTime {
    /// 创建初始时间 (frame=0, turn=0)
    pub fn new() -> Self {
        Self { frame: 0, turn: 0 }
    }

    /// 创建指定帧数和回合数的 GameTime
    pub fn at(frame: u64, turn: u64) -> Self {
        Self { frame, turn }
    }

    /// 当前帧数（单调递增）
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// 当前回合数（单调递增）
    pub fn turn(&self) -> u64 {
        self.turn
    }

    /// 推进一帧（游戏逻辑帧，非渲染帧）
    pub fn advance_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    /// 推进一回合（战斗回合/探索回合）
    pub fn advance_turn(&mut self) {
        self.turn = self.turn.saturating_add(1);
    }

    /// 设置当前帧数（用于回放还原）
    pub fn set_frame(&mut self, frame: u64) {
        self.frame = frame;
    }

    /// 设置当前回合数（用于回放还原）
    pub fn set_turn(&mut self, turn: u64) {
        self.turn = turn;
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F{}_T{}", self.frame, self.turn)
    }
}

#[cfg(test)]
mod tests;
