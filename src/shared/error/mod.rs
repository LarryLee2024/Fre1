//! 共享错误工具
//!
//! 提供 GameResult<T>, InfrastructureError, ErrorContext, LogIfError 等通用错误工具。
//! 从 src/core/error/game_result.rs 迁移至此（Phase 1.2）。
//!
//! 设计原则：
//! - ErrorContext: 为错误附加业务上下文，不转换错误类型（DEBUG 级别）
//! - LogIfError: 非关键路径的快速错误日志，返回 Option<T>（DEBUG 级别）
//! - InfrastructureError: 基础设施通用错误枚举，带错误码（INF001-INF004）
//! - GameResult<T>: InfrastructureError 的 Result 别名

mod context;
mod extensions;
mod result;

pub use context::ErrorContext;
pub use extensions::LogIfError;
pub use result::{GameResult, InfrastructureError};
