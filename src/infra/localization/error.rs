//! Localization 错误类型
//!
//! 零依赖层，仅使用 Rust 标准库。
//! 详见 `docs/03-technical/localization-design.md` 附录 A

use std::error::Error;
use std::fmt;

/// Locale ID: 语言标识符，遵循 BCP-47 格式。
///
/// 如 "en-US", "zh-CN", "ja-JP", "zz-ZZ"
pub type LocaleId = String;

/// Localization 错误类型
#[derive(Debug, Clone)]
pub enum LocError {
    /// Key 在重试所有 fallback locale 后仍未找到
    KeyNotFound {
        key: String,
        locale: LocaleId,
        fallbacks_attempted: Vec<LocaleId>,
    },

    /// 参数不匹配：pattern 需要某参数但未提供
    MissingParameter {
        key: String,
        missing: Vec<String>,
        provided: Vec<String>,
    },

    /// 内部解析错误（.ftl 语法异常）
    ParseError {
        file: Option<String>,
        line: Option<usize>,
        message: String,
    },

    /// 未分类内部错误
    Internal(String),
}

impl fmt::Display for LocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocError::KeyNotFound {
                key,
                locale,
                fallbacks_attempted,
            } => {
                write!(
                    f,
                    "Key '{}' not found in locale '{}' (fallbacks attempted: {:?})",
                    key, locale, fallbacks_attempted
                )
            }
            LocError::MissingParameter {
                key,
                missing,
                provided,
            } => {
                write!(
                    f,
                    "Key '{}' missing parameters: {:?} (provided: {:?})",
                    key, missing, provided
                )
            }
            LocError::ParseError {
                file,
                line,
                message,
            } => {
                if let (Some(file_path), Some(line_num)) = (file, line) {
                    write!(f, "Parse error at {}:{}: {}", file_path, line_num, message)
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            LocError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl Error for LocError {}
