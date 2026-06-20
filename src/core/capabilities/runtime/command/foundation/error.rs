//! Command 领域错误枚举。
//!
//! 定义业务命令处理过程中可能出现的错误类型。

/// Command 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CommandError {
    /// 命令队列已满
    #[error("command queue full (max {max})")]
    QueueFull { max: usize },
    /// 命令无效
    #[error("invalid command: {reason}")]
    InvalidCommand { reason: String },
    /// 命令执行失败
    #[error("execution failed for '{command}': {reason}")]
    ExecutionFailed {
        /// 命令名称
        command: String,
        /// 失败原因
        reason: String,
    },
}
