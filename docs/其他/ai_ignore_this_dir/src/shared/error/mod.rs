//! 共享错误工具
//!
//! 仅提供零外部依赖的错误工具 trait：ErrorContext、LogIfError。
//!
//! 设计原则（ADR-028）：
//! - ErrorContext: 为错误附加业务上下文，不转换错误类型（DEBUG 级别）
//! - LogIfError: 非关键路径的快速错误日志，返回 Option<T>（DEBUG 级别）
//!
//! 🟥 InfrastructureError 和 InfraResult 已迁移到 infrastructure::error

mod context;
mod extensions;

pub use context::ErrorContext;
pub use extensions::LogIfError;
