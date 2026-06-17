//! LocalizedTextCache：文本解析缓存

use std::collections::HashMap;
use std::collections::VecDeque;

use super::locale::Locale;

/// 文本解析缓存（LRU 淘汰策略）
/// key: (ftl_key, locale, args_hash) → formatted_string
///
/// 使用 VecDeque 作为访问顺序队列，O(1) 头尾操作。
/// get/insert 时将 key 移至队尾（最近使用），淘汰时从队头移除（最久未用）。
#[derive(Debug, bevy::prelude::Resource)]
pub struct LocalizedTextCache {
    cache: HashMap<(String, Locale, u64), String>,
    /// 缓存上限
    max_entries: usize,
    /// 访问顺序队列（VecDeque，队头=最久未用，队尾=最近使用）
    access_order: VecDeque<(String, Locale, u64)>,
}

impl Default for LocalizedTextCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizedTextCache {
    /// 创建缓存（默认上限 2048 条）
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            max_entries: 2048,
        }
    }

    /// 创建指定上限的缓存
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_entries),
            access_order: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    /// 获取缓存的格式化结果
    pub fn get(&mut self, key: &str, locale: &Locale, args_hash: u64) -> Option<String> {
        let cache_key = (key.to_string(), *locale, args_hash);
        if let Some(value) = self.cache.get(&cache_key) {
            // O(n) 查找并移除旧位置，O(1) push_back 到队尾
            self.access_order.retain(|k| k != &cache_key);
            self.access_order.push_back(cache_key);
            return Some(value.clone());
        }
        None
    }

    /// 插入缓存条目
    pub fn insert(&mut self, key: &str, locale: &Locale, args_hash: u64, value: String) {
        let cache_key = (key.to_string(), *locale, args_hash);

        // 如果已存在，更新访问顺序
        if self.cache.contains_key(&cache_key) {
            self.access_order.retain(|k| k != &cache_key);
            self.access_order.push_back(cache_key.clone());
            self.cache.insert(cache_key, value);
            return;
        }

        // LRU 淘汰：从队头移除最久未用的条目
        while self.cache.len() >= self.max_entries {
            if let Some(oldest) = self.access_order.pop_front() {
                self.cache.remove(&oldest);
            } else {
                break;
            }
        }

        self.cache.insert(cache_key.clone(), value);
        self.access_order.push_back(cache_key);
    }

    /// 清空缓存（语言切换时调用）
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_insert_and_get() {
        let mut cache = LocalizedTextCache::new();
        cache.insert("hello", &Locale::ZhCn, 0, "你好".to_string());

        let result = cache.get("hello", &Locale::ZhCn, 0);
        assert_eq!(result, Some("你好".to_string()));
    }

    #[test]
    fn cache_miss() {
        let mut cache = LocalizedTextCache::new();
        let result = cache.get("hello", &Locale::ZhCn, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn cache_lru_eviction() {
        let mut cache = LocalizedTextCache::with_capacity(2);

        cache.insert("a", &Locale::ZhCn, 0, "A".to_string());
        cache.insert("b", &Locale::ZhCn, 0, "B".to_string());
        // 触发淘汰
        cache.insert("c", &Locale::ZhCn, 0, "C".to_string());

        assert_eq!(cache.len(), 2);
        assert!(cache.get("a", &Locale::ZhCn, 0).is_none());
        assert_eq!(cache.get("b", &Locale::ZhCn, 0), Some("B".to_string()));
        assert_eq!(cache.get("c", &Locale::ZhCn, 0), Some("C".to_string()));
    }

    #[test]
    fn cache_clear() {
        let mut cache = LocalizedTextCache::new();
        cache.insert("hello", &Locale::ZhCn, 0, "你好".to_string());
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn cache_args_hash_different() {
        let mut cache = LocalizedTextCache::new();
        cache.insert("test", &Locale::ZhCn, 100, "v1".to_string());
        cache.insert("test", &Locale::ZhCn, 200, "v2".to_string());

        assert_eq!(
            cache.get("test", &Locale::ZhCn, 100),
            Some("v1".to_string())
        );
        assert_eq!(
            cache.get("test", &Locale::ZhCn, 200),
            Some("v2".to_string())
        );
    }

    #[test]
    fn cache_lru_access_refreshes_order() {
        let mut cache = LocalizedTextCache::with_capacity(2);
        cache.insert("a", &Locale::ZhCn, 0, "A".to_string());
        cache.insert("b", &Locale::ZhCn, 0, "B".to_string());
        // 访问 "a"，使其变为最近使用
        cache.get("a", &Locale::ZhCn, 0);
        // 插入 "c"，应淘汰 "b"（最久未用）
        cache.insert("c", &Locale::ZhCn, 0, "C".to_string());

        assert!(cache.get("b", &Locale::ZhCn, 0).is_none());
        assert_eq!(cache.get("a", &Locale::ZhCn, 0), Some("A".to_string()));
        assert_eq!(cache.get("c", &Locale::ZhCn, 0), Some("C".to_string()));
    }
}
