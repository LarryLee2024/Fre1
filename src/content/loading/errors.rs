//! Content 加载管线的错误类型

use std::path::PathBuf;

/// 配置加载错误。
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// 文件读取失败。
    FileReadError { path: PathBuf, reason: String },
    /// RON 反序列化失败。
    DeserializeError { path: PathBuf, detail: String },
    /// Definition 转换失败。
    ConversionError { path: PathBuf, detail: String },
    /// 加载后校验失败。
    ValidationFailed {
        path: PathBuf,
        errors: Vec<ValidationError>,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileReadError { path, reason } => {
                write!(f, "failed to read {}: {}", path.display(), reason)
            }
            ConfigError::DeserializeError { path, detail } => {
                write!(f, "failed to deserialize {}: {}", path.display(), detail)
            }
            ConfigError::ConversionError { path, detail } => {
                write!(f, "failed to convert {}: {}", path.display(), detail)
            }
            ConfigError::ValidationFailed { path, errors } => {
                write!(
                    f,
                    "validation failed for {}: {} error(s)",
                    path.display(),
                    errors.len()
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Definition 校验错误。
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// ID 为空。
    EmptyId,
    /// ID 前缀不匹配。
    InvalidIdPrefix { id: String, expected_prefix: String },
    /// ID 格式非法（非数字后缀）。
    InvalidIdFormat { id: String, detail: String },
    /// 必填字段缺失。
    MissingField { field: String },
    /// 数值超出合法范围。
    OutOfRange {
        field: String,
        value: f64,
        min: f64,
        max: f64,
    },
    /// 引用了不存在的 Definition。
    BrokenReference {
        field: String,
        referenced_id: String,
    },
    /// 自定义校验错误。
    Custom(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyId => write!(f, "definition ID is empty"),
            ValidationError::InvalidIdPrefix {
                id,
                expected_prefix,
            } => {
                write!(
                    f,
                    "ID '{}' does not start with expected prefix '{}'",
                    id, expected_prefix
                )
            }
            ValidationError::InvalidIdFormat { id, detail } => {
                write!(f, "ID '{}' has invalid format: {}", id, detail)
            }
            ValidationError::MissingField { field } => {
                write!(f, "required field '{}' is missing", field)
            }
            ValidationError::OutOfRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "field '{}' value {} is out of range [{}, {}]",
                    field, value, min, max
                )
            }
            ValidationError::BrokenReference {
                field,
                referenced_id,
            } => {
                write!(
                    f,
                    "field '{}' references non-existent definition '{}'",
                    field, referenced_id
                )
            }
            ValidationError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}
