//! 共享错误工具
//!
//! 提供 GameResult<T>, InfrastructureError, ErrorContext, LogIfError 等通用错误工具。
//! 从 src/core/error/game_result.rs 迁移至此（Phase 1.2）。

mod result;

pub use result::{GameResult, InfrastructureError};
