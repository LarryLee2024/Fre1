//! OnceGuard 单元测试
//!
//! 验证一次性守卫只在第一次调用时触发，后续调用返回 false。

use crate::infra::logging::rate_limit::OnceGuard;

#[test]
fn once_guard_fires_only_once() {
    let guard = OnceGuard::new();
    assert!(guard.try_fire());
    assert!(!guard.try_fire());
    assert!(!guard.try_fire());
}
