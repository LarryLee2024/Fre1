//! LocalizationDatabase — 核心本地化文本数据库
//!
//! 全局唯一 ECS Resource，存储所有 locale 的 key→value 映射，
//! 提供三级回退链解析能力。
//!
//! 详见 `docs/03-technical/localization-design.md` §3

use std::collections::HashMap;

use bevy::prelude::*;

use super::error::{LocError, LocaleId};

/// 解析后的 Pattern，含原始文本和预提取的变量名列表
#[derive(Debug, Clone, Reflect)]
pub struct Pattern {
    /// 原始模式文本（带 {$var} 占位符）
    pub template: String,
    /// 从 template 中提取的变量名（按出现顺序）
    pub variables: Vec<String>,
}

/// 核心 Localization 数据库，全局唯一 ECS Resource
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct LocalizationDatabase {
    /// 当前 locale
    current_locale: LocaleId,
    /// 按 (locale, key) 索引的原始 pattern 映射
    patterns: HashMap<LocaleId, HashMap<String, Pattern>>,
}

impl Default for LocalizationDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizationDatabase {
    /// 创建空数据库（默认 locale = en-US）
    pub fn new() -> Self {
        Self {
            current_locale: "en-US".into(),
            patterns: HashMap::new(),
        }
    }

    /// 为指定 locale 批量插入 pattern
    pub fn load_patterns(&mut self, locale: &LocaleId, entries: HashMap<String, Pattern>) {
        let locale_entry = self.patterns.entry(locale.clone()).or_default();
        locale_entry.extend(entries);
    }

    /// 切换当前语言。
    ///
    /// 调用方应同时清除缓存（通过 `detect_locale_change_and_clear_cache` 系统自动处理）。
    pub fn set_locale(&mut self, locale: LocaleId) {
        self.current_locale = locale.clone();
    }

    /// 获取当前 locale
    pub fn current_locale(&self) -> &LocaleId {
        &self.current_locale
    }

    /// 获取指定 locale 的 pattern（内部辅助方法）
    fn get_pattern(&self, locale: &str, key: &str) -> Option<&Pattern> {
        self.patterns.get(locale).and_then(|m| m.get(key))
    }

    /// 格式化 pattern：将 {$var} 替换为参数值
    fn format_pattern(&self, pattern: &Pattern, params: &[(&str, &str)]) -> String {
        if pattern.variables.is_empty() {
            return pattern.template.clone();
        }
        let mut result = pattern.template.clone();
        for (name, value) in params {
            let placeholder = format!("{{${}}}", name);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// 解析文本 —— 三级回退链
    ///
    /// # 回退链
    /// 1. 当前 locale → 有 pattern → 解析后返回
    /// 2. 当前 locale → 无 pattern → fallback 到 "en-US"
    /// 3. en-US → 无 pattern → 返回 raw_key 字符串（兜底）
    ///
    /// # 参数
    /// - `key`: 完整的 LocalizationKey，如 "ability.abl_000042.name"
    /// - `params`: 插值参数。无参数时传空 slice
    pub fn resolve(&self, key: &str, params: &[(&str, &str)]) -> Result<String, LocError> {
        let current = self.current_locale.as_str();

        // Step 1: 尝试当前 locale
        if let Some(pattern) = self.get_pattern(current, key) {
            return Ok(self.format_pattern(pattern, params));
        }

        // Step 2: Fallback 到 en-US
        if current != "en-US"
            && let Some(pattern) = self.get_pattern("en-US", key)
        {
            return Ok(self.format_pattern(pattern, params));
        }

        // Step 3: 兜底 — 返回 key 本身
        // key 本身是开发者能理解的描述性字符串
        Ok(key.to_string())
    }

    /// 带缓存的 resolve — 推荐 UI 系统使用
    pub fn resolve_cached(
        &self,
        key: &str,
        params: &[(&str, &str)],
        cache: &mut super::cache::LocalizedTextCache,
    ) -> Result<String, LocError> {
        let locale = &self.current_locale;

        // 尝试缓存
        if let Some(cached) = cache.get(locale, key, params) {
            return Ok(cached.to_string());
        }

        // 缓存未命中 → 解析
        let result = self.resolve(key, params)?;

        // 写入缓存（仅缓存成功解析的结果）
        cache.set(locale, key, params, result.clone());

        Ok(result)
    }

    /// 检查 key 在指定 locale 中是否存在
    pub fn has_key(&self, locale: &str, key: &str) -> bool {
        self.patterns
            .get(locale)
            .is_some_and(|m| m.contains_key(key))
    }

    /// 获取指定 locale 的所有 key
    pub fn all_keys(&self, locale: &str) -> Vec<&str> {
        self.patterns
            .get(locale)
            .map(|m| m.keys().map(|k| k.as_str()).collect())
            .unwrap_or_default()
    }

    /// 获取当前 locale 的所有缺失 key（相对 en-US）
    pub fn missing_keys(&self) -> Vec<&str> {
        let Some(en) = self.patterns.get("en-US") else {
            return vec![];
        };
        let current = self.patterns.get(&self.current_locale);
        en.keys()
            .filter(|k| !current.is_some_and(|m| m.contains_key(*k)))
            .map(|k| k.as_str())
            .collect()
    }

    /// 覆盖率: 当前 locale 相对 en-US 的翻译完成度
    pub fn coverage(&self) -> f64 {
        let Some(en) = self.patterns.get("en-US") else {
            return 1.0;
        };
        let Some(current) = self.patterns.get(&self.current_locale) else {
            return 0.0;
        };
        if en.is_empty() {
            return 1.0;
        }
        let translated = en.keys().filter(|k| current.contains_key(*k)).count();
        translated as f64 / en.len() as f64
    }

    /// 获取所有已加载的 locale 列表
    pub fn loaded_locales(&self) -> Vec<&str> {
        self.patterns.keys().map(|k| k.as_str()).collect()
    }

    /// 获取指定 locale 的 pattern 总数
    pub fn pattern_count(&self, locale: &str) -> usize {
        self.patterns.get(locale).map_or(0, |m| m.len())
    }
}
