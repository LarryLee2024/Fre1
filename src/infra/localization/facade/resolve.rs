//! resolve_cached — 跨层调用的本地化解析（带缓存）
//!
//! 组合 database (resolve) 和 cache (get/set) 的解耦入口。

use crate::infra::localization::foundation::LocError;
use crate::infra::localization::storage::{LocalizationDatabase, LocalizedTextCache};

/// 带缓存的本地化解析
///
/// 先查缓存，未命中则调用 `db.resolve()` 并回写缓存。
/// 推荐 UI 渲染系统使用此函数代替直接调用 `db.resolve()`。
pub fn resolve_cached(
    db: &LocalizationDatabase,
    cache: &mut LocalizedTextCache,
    key: &str,
    params: &[(&str, &str)],
) -> Result<String, LocError> {
    let locale = db.current_locale();

    // 尝试缓存
    if let Some(cached) = cache.get(locale, key, params) {
        return Ok(cached.to_string());
    }

    // 缓存未命中 -> 解析
    let result = db.resolve(key, params)?;

    // 写入缓存（仅缓存成功解析的结果）
    cache.set(locale, key, params, result.clone());

    Ok(result)
}
