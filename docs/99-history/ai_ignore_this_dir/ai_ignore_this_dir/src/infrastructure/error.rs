/// 基础设施层通用错误
///
/// 从 shared/error/result.rs 迁移至此（ADR-028）
/// 共享层只保留 ErrorContext、LogIfError 等零依赖工具 trait。
///
/// 使用方式：
/// - 基础设施层使用 InfraResult<T> 封装底层 IO/序列化错误
/// - 🟥 禁止：业务层使用 InfraResult<T> 返回业务错误
use thiserror::Error;

/// 基础设施层通用错误
///
/// 只收纳基础设施通用错误，不包含任何业务错误变体。
/// 错误码格式：INF + 三位序号
#[derive(Error, Debug)]
pub enum InfrastructureError {
    /// 资源加载失败
    #[error("INF001: 资源加载失败: {0}")]
    AssetLoadFailed(String),

    /// 序列化/反序列化失败
    #[error("INF002: 序列化错误: {0}")]
    SerializationError(String),

    /// IO 错误
    #[error("INF003: IO 错误: {0}")]
    IoError(String),

    /// 存档损坏
    #[error("INF004: 存档损坏: {0}")]
    SaveDataCorrupted(String),
}

/// 基础设施层通用结果类型
///
/// 仅用于基础设施层（资源加载、存档 IO、序列化等）。
/// 🟥 禁止用于业务层错误返回。
pub type InfraResult<T> = Result<T, InfrastructureError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infrastructure_error_消息格式() {
        let err = InfrastructureError::AssetLoadFailed("units/player.ron".to_string());
        assert!(err.to_string().contains("INF001"));
        assert!(err.to_string().contains("units/player.ron"));
    }

    #[test]
    fn infra_result_类型可用() {
        let result: InfraResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn infra_result_err_可转换() {
        let result: InfraResult<i32> =
            Err(InfrastructureError::IoError("file not found".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("INF003"));
    }
}
