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
    #[error("formula '{formula_id}' 未找到: {detail}")]
    FormulaNotFound { formula_id: String, detail: String },
    /// ExecutionContext 数据缺失（不变量 3.3）
    #[error("context 字段 '{field}' 缺失: {detail}")]
    ContextMissing { field: String, detail: String },
    /// 计算结果数值非法（不变量 3.4）
    #[error("无效的结果: {0}")]
    InvalidResult(String),
    /// 自定义计算未注册（不变量 3.5）
    #[error("自定义 execution '{0}' 未注册")]
    CustomExecutionNotRegistered(String),
    /// 不支持的执行类型
    #[error("不支持的 execution 类型: {0}")]
    UnsupportedExecutionType(String),
    /// 通用运行时错误
    #[error("运行时错误: {0}")]
    Runtime(String),
}
