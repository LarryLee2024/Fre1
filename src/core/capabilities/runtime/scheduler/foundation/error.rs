//! Scheduler 领域错误枚举。
//!
//! 定义调度器操作过程中可能出现的错误类型。

use super::types::TickPhase;

/// Scheduler 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SchedulerError {
    /// 调度器未初始化
    #[error("scheduler 未初始化")]
    NotInitialized,
    /// 调度器已暂停
    #[error("scheduler 已暂停")]
    Paused,
    /// 无效的阶段转换
    #[error("无效的 phase 转换: {from:?} → {to:?}")]
    InvalidTransition {
        /// 来源阶段
        from: TickPhase,
        /// 目标阶段
        to: TickPhase,
    },
    /// 帧计数器溢出
    #[error("帧计数器在 {frame} 溢出")]
    FrameOverflow { frame: u64 },
}
