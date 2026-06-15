//! ErrorContext trait
//!
//! 为 Result 增加错误上下文信息的工具 trait。
//! 适用场景：底层错误需要附加业务上下文时使用。
//!
//! 示例：
//! ```ignore
//! let file = std::fs::read_to_string("config.ron")
//!     .with_context(|| "读取技能配置失败".to_string())?;
//! ```
//!
//! 🟥 禁止用于包装领域错误（领域错误应自带完整上下文）
//! 🟥 仅输出 DEBUG 级别日志辅助调试，不改变错误类型

use std::fmt::Display;

use tracing::debug;

/// 为 Result 增加错误上下文信息的 trait
///
/// 当底层错误需要附加高层上下文时使用。
/// 不转换错误类型，仅记录上下文到 DEBUG 日志。
///
/// 通用实现覆盖所有 `Result<T, E: Display>`，
/// 包括 InfrastructureError（它通过 thiserror 实现了 Display）。
pub trait ErrorContext<T, E> {
    /// 为错误附加上下文描述
    ///
    /// `f` 是惰性求值的闭包，仅在 Err 时调用。
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T, E>;
}

/// 为任意 `Result<T, E: Display>` 实现 ErrorContext
impl<T, E: Display> ErrorContext<T, E> for Result<T, E> {
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T, E> {
        self.map_err(|e| {
            let context = f();
            debug!(
                target: "error",
                error = %e,
                context = %context,
                "操作失败"
            );
            e
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::error::InfrastructureError;

    /// InfrastructureError 的 with_context 应输出 DEBUG 日志并保留原始错误
    #[test]
    fn infrastructure_error_带上下文() {
        let result: Result<i32, InfrastructureError> =
            Err(InfrastructureError::AssetLoadFailed("test.ron".into()));

        let err = result
            .with_context(|| "加载测试配置".to_string())
            .unwrap_err();

        assert!(err.to_string().contains("INF001"));
        assert!(err.to_string().contains("test.ron"));
    }

    /// Ok 路径应直接通过，不触发闭包
    #[test]
    fn ok_with_context_透传() {
        let result: Result<i32, InfrastructureError> = Ok(42);

        let val = result
            .with_context(|| panic!("闭包不应在 Ok 时被调用"))
            .unwrap();

        assert_eq!(val, 42);
    }

    /// 字符串错误类型的 with_context
    #[test]
    fn string_error_带上下文() {
        let result: Result<i32, &str> = Err("文件不存在");

        let err = result
            .with_context(|| "读取配置失败".to_string())
            .unwrap_err();

        assert_eq!(err, "文件不存在");
    }
}
