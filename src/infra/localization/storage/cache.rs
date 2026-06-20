//! LocalizedTextCache — 运行时解析文本缓存
//!
//! 缓存解析后的文本，避免重复调用 resolve()。
//! 语言切换或热重载时全量失效。
//!
//! 详见 `docs/03-technical/localization-design.md` §11.1

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use crate::infra::localization::foundation::LocaleId;

/// 运行时解析文本缓存
///
/// # 失效策略
/// - `set_locale()` 时全量失效（清空）
/// - 单个 .ftl 热重载时相关 key 失效
///
/// # 线程安全
/// 只在主线程访问，不需要 Arc/RwLock
#[derive(Resource, Default)]
pub struct LocalizedTextCache {
    /// cache[ (locale, key, params_hash) ] = resolved_text
    /// params_hash 用于区分同一 key 不同参数的情况
    cache: HashMap<(LocaleId, String, u64), String>,
}

impl LocalizedTextCache {
    /// 最大缓存条目数，防止内存无限增长
    const MAX_ENTRIES: usize = 500;

    /// 尝试从缓存获取
    pub fn get(&self, locale: &LocaleId, key: &str, params: &[(&str, &str)]) -> Option<&str> {
        let params_hash = params_hash(params);
        self.cache
            .get(&(locale.clone(), key.to_string(), params_hash))
            .map(|s| s.as_str())
    }

    /// 写入缓存
    pub fn set(&mut self, locale: &LocaleId, key: &str, params: &[(&str, &str)], resolved: String) {
        // 容量超限时执行驱逐
        if self.cache.len() >= Self::MAX_ENTRIES {
            // 简单策略: 超限时清空 1/4 的条目
            let remove_count = Self::MAX_ENTRIES / 4;
            let keys: Vec<_> = self.cache.keys().take(remove_count).cloned().collect();
            for k in keys {
                self.cache.remove(&k);
            }
        }

        let params_hash = params_hash(params);
        self.cache
            .insert((locale.clone(), key.to_string(), params_hash), resolved);
    }

    /// 全量失效（locale 切换时调用）
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 当前缓存条目数
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// 计算参数的哈希（用于区分同一 key 不同参数）
fn params_hash(params: &[(&str, &str)]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    params.len().hash(&mut hasher);
    for (k, v) in params {
        k.hash(&mut hasher);
        v.hash(&mut hasher);
    }
    hasher.finish()
}

/// 系统: 检测 locale 变化并清理缓存
///
/// 通过比较 current_locale 与上次记录的值来检测切换。
pub fn detect_locale_change_and_clear_cache(
    db: Res<super::database::LocalizationDatabase>,
    mut last_locale: Local<Option<LocaleId>>,
    mut cache: ResMut<LocalizedTextCache>,
) {
    let current = db.current_locale().clone();
    if last_locale.as_ref() != Some(&current) {
        let _ = last_locale.insert(current);
        cache.clear();
    }
}
