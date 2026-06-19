//! OnceGuard — 基于 AtomicBool 的一次性日志守卫
//!
//! 用于日志风暴保护：每个调用点只触发一次日志输出。
//! 配合 `warn_once!` / `error_once!` 宏使用。
//!
//! 详见 ADR-052

use std::sync::atomic::{AtomicBool, Ordering};

/// 基于 AtomicBool 的一次性日志守卫。
///
/// 每个调用点只触发一次日志输出，防止日志风暴。
///
/// # 用法
///
/// ```ignore
/// static GUARD: OnceGuard = OnceGuard::new();
/// warn_once!(GUARD, code = ?LogCode::XXX, "message");
/// ```
pub struct OnceGuard {
    fired: AtomicBool,
}

impl OnceGuard {
    /// 创建新的守卫实例。
    pub const fn new() -> Self {
        Self {
            fired: AtomicBool::new(false),
        }
    }

    /// 尝试触发：首次调用返回 `true`，后续返回 `false`。
    pub fn try_fire(&self) -> bool {
        self.fired
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_guard_fires_only_once() {
        let guard = OnceGuard::new();
        assert!(guard.try_fire());
        assert!(!guard.try_fire());
        assert!(!guard.try_fire());
    }
}
