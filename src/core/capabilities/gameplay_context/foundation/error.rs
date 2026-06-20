//! GameplayContext 构建错误。
//!
//! 定义上下文构建过程中的各类校验错误。

/// 上下文构建错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContextBuildError {
    /// 缺失必填字段（列出缺失字段名）
    #[error("missing required fields: {0:?}")]
    MissingFields(Vec<String>),
    /// 溯源链检测到循环
    #[error("context chain cycle detected")]
    CycleDetected,
    /// 溯源链达到长度上限
    #[error("chain length {current} exceeds max {max}")]
    ChainTooLong { current: u8, max: u8 },
}
