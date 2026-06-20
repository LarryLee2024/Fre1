//! Stacking 领域错误。
//!
//! 定义堆叠判定过程中的各类错误。
//!
//! 详见 docs/02-domain/capabilities/stacking_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/stacking_schema.md §3。

/// Stacking 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StackingError {
    /// 无效的堆叠配置（如 Aggregate 但 max_stacks < 2）
    #[error("invalid stacking config: {0}")]
    InvalidConfig(String),
    /// 堆叠标识不匹配
    #[error("identity mismatch: '{existing_def_id}' vs '{incoming_def_id}': {detail}")]
    IdentityMismatch {
        existing_def_id: String,
        incoming_def_id: String,
        detail: String,
    },
    /// 运行时错误
    #[error("stacking runtime error: {0}")]
    Runtime(String),
}
