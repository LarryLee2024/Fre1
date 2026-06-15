//! LogIfError trait
//!
//! 为 Result 添加便捷的 DEBUG 级别日志记录能力。
//! 适用场景：非关键路径的错误只需记录，不需要冒泡处理。
//!
//! 🟥 禁止用于核心业务路径（核心路径错误必须显式处理）
//! 🟥 仅输出 DEBUG 级别，不影响 INFO 级别核心日志

use std::fmt::Display;

use tracing::debug;

/// 为 Result 添加便捷的日志记录能力
///
/// 当你不关心错误的传播，只想在出错时记录一行 DEBUG 日志时使用。
///
/// # 适用场景
///
/// - 非关键路径：缓存未命中、可选配置缺失、指标采集失败
/// - 调试阶段：快速了解哪些路径出了错，再决定是否需要显式处理
///
/// # 禁止场景
///
/// - 核心业务路径（伤害计算、状态变更等必须显式处理错误）
/// - 需要向上传播错误的场景（应使用 `?` 运算符）
///
/// # 示例
///
/// ```ignore
/// // 非关键路径：加载可选的用户设置
/// let settings = load_user_settings()
///     .log_if_error("加载用户设置失败，使用默认值")
///     .unwrap_or_default();
///
/// // 关键路径：必须显式处理
/// let damage = calculate_damage(attacker, defender)?;  // ✅ 正确
/// let damage = calculate_damage(attacker, defender)
///     .log_if_error("伤害计算失败");  // ❌ 禁止
/// ```
pub trait LogIfError<T, E: Display> {
    /// 如果 Result 是 Err，输出 DEBUG 日志并返回 None
    /// 如果 Result 是 Ok，返回 Some(value)
    fn log_if_error(self, context: &str) -> Option<T>;
}

/// 为任意 `Result<T, E: Display>` 实现 LogIfError
impl<T, E: Display> LogIfError<T, E> for Result<T, E> {
    fn log_if_error(self, context: &str) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(e) => {
                debug!(
                    target: "error",
                    error = %e,
                    context = %context,
                    "操作失败（已忽略）"
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ok 路径应返回 Some 且不输出日志
    #[test]
    fn ok_returns_some() {
        let result: Result<i32, &str> = Ok(42);
        let val = result.log_if_error("测试");
        assert_eq!(val, Some(42));
    }

    /// Err 路径应返回 None
    #[test]
    fn err_returns_none() {
        let result: Result<i32, &str> = Err("出错了");
        let val = result.log_if_error("测试错误");
        assert!(val.is_none());
    }

    /// 不同类型错误的兼容性
    #[test]
    fn works_with_different_error_types() {
        // &str 错误
        let r1: Result<i32, &str> = Err("str error");
        assert!(r1.log_if_error("str").is_none());

        // String 错误
        let r2: Result<i32, String> = Err("string error".into());
        assert!(r2.log_if_error("string").is_none());

        // 自定义 Display 类型
        #[derive(Debug)]
        struct CustomError(i32);
        impl Display for CustomError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "自定义错误 {}", self.0)
            }
        }
        let r3: Result<i32, CustomError> = Err(CustomError(42));
        let val = r3.log_if_error("custom");
        assert!(val.is_none());
    }

    /// 链式调用：log_if_error 后跟 unwrap_or_default
    #[test]
    fn chaining_with_unwrap_or_default() {
        let result: Result<i32, &str> = Err("丢失");
        let val = result.log_if_error("链式测试").unwrap_or(-1);
        assert_eq!(val, -1);
    }
}
