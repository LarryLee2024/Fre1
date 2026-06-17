/// 资源加载领域错误
///
/// 覆盖资产加载过程中的预期异常：文件缺失、格式错误等。
use thiserror::Error;

/// 资产加载错误
///
/// 错误码格式：AST + 三位序号
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AssetError {
    /// AST001: 文件未找到
    #[error("AST001: 文件未找到: {path}")]
    FileNotFound { path: String },

    /// AST002: 格式解析错误
    #[error("AST002: 格式解析错误: {path}: {detail}")]
    ParseError { path: String, detail: String },

    /// AST003: 资源加载超时
    #[error("AST003: 资源加载超时: {path}")]
    LoadTimeout { path: String },
}

/// 资产加载结果类型
pub type AssetResult<T> = Result<T, AssetError>;
