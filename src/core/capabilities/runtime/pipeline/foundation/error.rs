//! Pipeline 领域错误枚举。
//!
//! 定义执行管线过程中可能出现的错误类型。

/// Pipeline 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PipelineError {
    /// 阶段未找到
    #[error("stage '{stage}' 未找到")]
    StageNotFound { stage: String },
    /// 步骤执行失败
    #[error("stage '{stage}' 中的 step '{step}' 执行失败: {detail}")]
    StepFailed {
        /// 阶段名称
        stage: String,
        /// 步骤名称
        step: String,
        /// 错误详情
        detail: String,
    },
    /// 管线被中止
    #[error("pipeline 已中止: {reason}")]
    Aborted { reason: String },
    /// 上下文数据缺失
    #[error("缺少 context key: {key}")]
    MissingContext { key: String },
}
