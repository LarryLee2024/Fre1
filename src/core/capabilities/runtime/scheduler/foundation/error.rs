//! Scheduler 领域错误枚举。
//!
//! 定义调度器操作过程中可能出现的错误类型。

use super::types::TickPhase;

/// Scheduler 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SchedulerError {
    /// 调度器未初始化
    #[error("scheduler not initialized")]
    NotInitialized,
    /// 调度器已暂停
    #[error("scheduler is paused")]
    Paused,
    /// 无效的阶段转换
    #[error("invalid phase transition: {from:?} \u{2192} {to:?}")]
    InvalidTransition {
        /// 来源阶段
        from: TickPhase,
        /// 目标阶段
        to: TickPhase,
    },
    /// 帧计数器溢出
    #[error("frame counter overflow at {frame}")]
    FrameOverflow { frame: u64 },
}
