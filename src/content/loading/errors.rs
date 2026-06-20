//! Content 加载管线的错误类型

/// 配置加载错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    /// 文件读取失败。
    #[error("failed to read {path}: {reason}")]
    FileReadError { path: String, reason: String },
    /// RON 反序列化失败。
    #[error("failed to deserialize {path}: {detail}")]
    DeserializeError { path: String, detail: String },
    /// Definition 转换失败。
    #[error("failed to convert {path}: {detail}")]
    ConversionError { path: String, detail: String },
    /// 加载后校验失败。
    #[error("validation failed for {path}: {} error(s)", errors.len())]
    ValidationFailed {
        path: String,
        errors: Vec<ValidationError>,
    },
}

/// Definition 校验错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    /// ID 为空。
    #[error("definition ID is empty")]
    EmptyId,
    /// ID 前缀不匹配。
    #[error("ID '{id}' does not start with expected prefix '{expected_prefix}'")]
    InvalidIdPrefix { id: String, expected_prefix: String },
    /// ID 格式非法（非数字后缀）。
    #[error("ID '{id}' has invalid format: {detail}")]
    InvalidIdFormat { id: String, detail: String },
    /// 必填字段缺失。
    #[error("required field '{field}' is missing")]
    MissingField { field: String },
    /// 数值超出合法范围。
    #[error("field '{field}' value {value} is out of range [{min}, {max}]")]
    OutOfRange {
        field: String,
        value: f64,
        min: f64,
        max: f64,
    },
    /// 引用了不存在的 Definition。
    #[error("field '{field}' references non-existent definition '{referenced_id}'")]
    BrokenReference {
        field: String,
        referenced_id: String,
    },
    /// 自定义校验错误。
    #[error("{0}")]
    Custom(String),
}
