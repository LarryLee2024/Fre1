//! 数据库测试 — 三级回退行为。
//!
//! 验证 resolve() 回退链：当前区域设置 -> en-US -> 原始键。

use crate::infra::localization::foundation::LocaleId;
use crate::infra::localization::storage::database::LocalizationDatabase;

#[test]
fn test_current_locale_resolve() {
    // TODO[P3][Localization][2026-06-21]: implement with proper fixture data
}

#[test]
fn test_fallback_to_en_us() {
    // TODO[P3][Localization][2026-06-21]: implement with proper fixture data
}

#[test]
fn test_fallback_to_raw_key() {
    // TODO[P3][Localization][2026-06-21]: implement with proper fixture data
}
