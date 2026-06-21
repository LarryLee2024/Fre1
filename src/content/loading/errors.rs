//! Content 加载管线的错误类型

/// 配置加载错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    /// 文件读取失败。
    #[error("读取 {path} 失败: {reason}")]
    FileReadError { path: String, reason: String },
    /// RON 反序列化失败。
    #[error("反序列化 {path} 失败: {detail}")]
    DeserializeError { path: String, detail: String },
    /// Definition 转换失败。
    #[error("转换 {path} 失败: {detail}")]
    ConversionError { path: String, detail: String },
    /// 加载后校验失败。
    #[error("{path} 校验失败: {} 个错误", errors.len())]
    ValidationFailed {
        path: String,
        errors: Vec<ValidationError>,
    },
}

/// Definition 校验错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    /// ID 为空。
    #[error("definition ID 为空")]
    EmptyId,
    /// ID 前缀不匹配。
    #[error("ID '{id}' 不以期望前缀 '{expected_prefix}' 开头")]
    InvalidIdPrefix { id: String, expected_prefix: String },
    /// ID 格式非法（非数字后缀）。
    #[error("ID '{id}' 格式非法: {detail}")]
    InvalidIdFormat { id: String, detail: String },
    /// 必填字段缺失。
    #[error("必填字段 '{field}' 缺失")]
    MissingField { field: String },
    /// 数值超出合法范围。
    #[error("字段 '{field}' 的值 {value} 超出范围 [{min}, {max}]")]
    OutOfRange {
        field: String,
        value: f64,
        min: f64,
        max: f64,
    },
    /// 引用了不存在的 Definition。
    #[error("字段 '{field}' 引用了不存在的 definition '{referenced_id}'")]
    BrokenReference {
        field: String,
        referenced_id: String,
    },
    /// 自定义校验错误。
    #[error("{0}")]
    Custom(String),
}
