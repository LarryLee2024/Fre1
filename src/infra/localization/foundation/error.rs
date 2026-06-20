//! Localization 错误类型
//!
//! 零依赖层，仅使用 Rust 标准库和 thiserror。
//! 详见 `docs/03-technical/localization-design.md` 附录 A

use super::locale_id::LocaleId;

/// Localization 错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum LocError {
    /// Key 在重试所有 fallback locale 后仍未找到
    #[error(
        "Key '{key}' not found in locale '{locale}' (fallbacks attempted: {fallbacks_attempted:?})"
    )]
    KeyNotFound {
        key: String,
        locale: LocaleId,
        fallbacks_attempted: Vec<LocaleId>,
    },

    /// 参数不匹配：pattern 需要某参数但未提供
    #[error("Key '{key}' missing parameters: {missing:?} (provided: {provided:?})")]
    MissingParameter {
        key: String,
        missing: Vec<String>,
        provided: Vec<String>,
    },

    /// 内部解析错误（.ftl 语法异常）
    #[error("Parse error: {message}")]
    ParseError {
        file: Option<String>,
        line: Option<usize>,
        message: String,
    },

    /// 未分类内部错误
    #[error("Internal error: {0}")]
    Internal(String),
}
