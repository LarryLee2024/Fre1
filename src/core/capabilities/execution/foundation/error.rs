//! Execution 领域错误。
//!
//! 定义执行计算过程中的各类错误。
//!
//! 详见 docs/02-domain/capabilities/execution_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/execution_schema.md §3。

/// Execution 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExecutionError {
    /// 计算公式 ID 未注册（V1: formula_id 已注册）
    #[error("formula '{formula_id}' not found: {detail}")]
    FormulaNotFound { formula_id: String, detail: String },
    /// ExecutionContext 数据缺失（不变量 3.3）
    #[error("context field '{field}' missing: {detail}")]
    ContextMissing { field: String, detail: String },
    /// 计算结果数值非法（不变量 3.4）
    #[error("invalid result: {0}")]
    InvalidResult(String),
    /// 自定义计算未注册（不变量 3.5）
    #[error("custom execution '{0}' not registered")]
    CustomExecutionNotRegistered(String),
    /// 不支持的执行类型
    #[error("unsupported execution type: {0}")]
    UnsupportedExecutionType(String),
    /// 通用运行时错误
    #[error("runtime error: {0}")]
    Runtime(String),
}
