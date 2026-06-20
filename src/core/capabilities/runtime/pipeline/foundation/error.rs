//! Pipeline 领域错误枚举。
//!
//! 定义执行管线过程中可能出现的错误类型。

/// Pipeline 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PipelineError {
    /// 阶段未找到
    #[error("stage '{0}' not found")]
    StageNotFound(String),
    /// 步骤执行失败
    #[error("step '{step}' in stage '{stage}' failed: {detail}")]
    StepFailed {
        /// 阶段名称
        stage: String,
        /// 步骤名称
        step: String,
        /// 错误详情
        detail: String,
    },
    /// 管线被中止
    #[error("pipeline aborted: {0}")]
    Aborted(String),
    /// 上下文数据缺失
    #[error("missing context key: {0}")]
    MissingContext(String),
}
