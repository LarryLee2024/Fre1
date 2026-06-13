/// 通用游戏结果类型
///
/// ADR-004 §决策: 基础设施层薄封装
/// 提供 GameResult<T> 作为基础设施层的通用结果类型别名，
/// 避免业务层使用全局统一错误类型。
///
/// 使用方式：
/// - 业务领域使用自己定义的错误枚举 + Result<T, DomainError>
/// - 基础设施层使用 GameResult<T> 封装底层 IO/序列化错误
///
/// 🟥 禁止：业务层使用 GameResult<T> 返回业务错误
use thiserror::Error;

/// 基础设施层通用错误
///
/// 只收纳基础设施通用错误，不包含任何业务错误变体。
/// ADR-004 §14.9.1: 基础设施层薄封装
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
pub type GameResult<T> = Result<T, InfrastructureError>;

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
    fn game_result_类型可用() {
        let result: GameResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn game_result_err_可转换() {
        let result: GameResult<i32> =
            Err(InfrastructureError::IoError("file not found".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("INF003"));
    }
}
