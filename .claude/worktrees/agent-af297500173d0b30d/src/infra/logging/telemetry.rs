//! 可观测性门面（Observability Facade）——Observer 的唯一埋点接口。
//!
//! # 职责
//!
//! 封装日志 + 度量调用，自动从 LogCode 派生 target，消除 info!() 中的 target 重复。
//! Observer 调用 `emit_info!`/`emit_warn!`/`emit_debug!` 后不需要再手动调用 `metrics::record` 或写 `target`。
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::shared::diagnostics::LogCode;
//!
//! emit_info!(
//!     LogCode::PRG002,
//!     entity = ?entity,
//!     old = old_level,
//!     new = new_level,
//!     "角色升级",
//! );
//! ```

use crate::shared::diagnostics::LogCode;

/// 记录一次可观测性事件（仅 metrics，无日志输出）。
/// 在 emit_info!/emit_warn!/emit_debug! 宏内部自动调用，通常不需要直接使用。
pub fn emit(code: LogCode) {
    crate::infra::logging::metrics::record(code);
}

/// 统一 INFO 日志 + 度量入口。
/// 自动从 LogCode 派生 target，消除 info!() 中的 target 字面量重复。
///
/// # 用法
///
/// ```ignore
/// emit_info!(LogCode::PRG002, entity = ?e.entity, old = e.old_level, "角色升级");
/// ```
#[macro_export]
macro_rules! emit_info {
    ($code:expr, $($arg:tt)*) => {
        {
            $crate::infra::logging::telemetry::emit($code);
            ::tracing::info!(
                target: $code.target(),
                $($arg)*
            );
        }
    };
}

/// 统一 WARN 日志 + 度量入口。
/// 自动从 LogCode 派生 target。
#[macro_export]
macro_rules! emit_warn {
    ($code:expr, $($arg:tt)*) => {
        {
            $crate::infra::logging::telemetry::emit($code);
            ::tracing::warn!(
                target: $code.target(),
                $($arg)*
            );
        }
    };
}

/// 统一 DEBUG 日志 + 度量入口。
/// 自动从 LogCode 派生 target。
#[macro_export]
macro_rules! emit_debug {
    ($code:expr, $($arg:tt)*) => {
        {
            $crate::infra::logging::telemetry::emit($code);
            ::tracing::debug!(
                target: $code.target(),
                $($arg)*
            );
        }
    };
}
