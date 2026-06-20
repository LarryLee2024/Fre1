//! rate_limit — 日志风暴保护
//!
//! 提供 `OnceGuard` 和 `warn_once!` / `error_once!` 宏，
//! 防止高频事件导致日志爆炸。

mod once_guard;

#[cfg(test)]
mod tests;

pub use once_guard::OnceGuard;

/// 一次性警告日志：每个调用点只输出一次。
///
/// # 用法
///
/// ```ignore
/// static GUARD: OnceGuard = OnceGuard::new();
/// warn_once!(GUARD, code = ?LogCode::XXX, "message");
/// ```
#[macro_export]
macro_rules! warn_once {
    ($guard:expr, $($arg:tt)*) => {
        if $guard.try_fire() {
            tracing::warn!($($arg)*);
        }
    };
}

/// 一次性错误日志：每个调用点只输出一次。
///
/// # 用法
///
/// ```ignore
/// static GUARD: OnceGuard = OnceGuard::new();
/// error_once!(GUARD, code = ?LogCode::XXX, "message");
/// ```
#[macro_export]
macro_rules! error_once {
    ($guard:expr, $($arg:tt)*) => {
        if $guard.try_fire() {
            tracing::error!($($arg)*);
        }
    };
}
