//! Cue 领域错误。
//!
//! 定义表现信号分发过程中的各类错误。
//!
//! 详见 docs/02-domain/capabilities/cue_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/cue_schema.md §3。

/// Cue 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CueError {
    /// Cue 未找到
    #[error("cue '{0}' 未找到")]
    CueNotFound(String),
    /// 无效的参数
    #[error("无效的 cue 参数: {0}")]
    InvalidParams(String),
}
