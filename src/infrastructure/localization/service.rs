//! LocalizationService：封装 fluent-rs 的核心本地化服务

use std::collections::HashMap;
use std::sync::Mutex;

use fluent::{FluentArgs, FluentBundle, FluentResource};

use super::locale::Locale;
use super::cache::LocalizedTextCache;

/// 本地化错误
#[derive(Debug, thiserror::Error)]
pub enum LocalizationError {
    #[error("FTL 解析失败: {0}")]
    ParseError(String),

    #[error("Key 未找到: {key} (locale: {locale})")]
    KeyNotFound { key: String, locale: String },

    #[error("FTL 文件加载失败: {0}")]
    LoadError(String),
}

/// 本地化核心服务（内部可变性通过 Mutex 实现）
#[derive(bevy::prelude::Resource)]
pub struct LocalizationService {
    inner: Mutex<LocalizationInner>,
}

struct LocalizationInner {
    /// 已加载的 FTL 内容（locale → 内容列表）
    ftl_contents: HashMap<Locale, Vec<String>>,
    /// 默认语言
    default_locale: Locale,
    /// 文本解析缓存
    cache: LocalizedTextCache,
}

impl LocalizationService {
    /// 创建空的 LocalizationService
    pub fn new(default_locale: Locale) -> Self {
        Self {
            inner: Mutex::new(LocalizationInner {
                ftl_contents: HashMap::new(),
                default_locale,
                cache: LocalizedTextCache::new(),
            }),
        }
    }

    /// 加载指定 locale 的 FTL 文件内容
    pub fn load_ftl(
        &self,
        locale: Locale,
        ftl_content: &str,
    ) -> Result<(), LocalizationError> {
        // 验证 FTL 内容可解析
        let _resource = FluentResource::try_new(ftl_content.to_string())
            .map_err(|(_, e)| LocalizationError::ParseError(format!("{:?}", e)))?;

        let mut inner = self.inner.lock().map_err(|e| {
            LocalizationError::LoadError(format!("锁获取失败: {}", e))
        })?;

        inner
            .ftl_contents
            .entry(locale)
            .or_default()
            .push(ftl_content.to_string());

        Ok(())
    }

    /// 解析本地化 Key，返回目标语言文本
    pub fn resolve(
        &self,
        key: &str,
        locale: &Locale,
        args: Option<&FluentArgs>,
    ) -> String {
        let mut inner = match self.inner.lock() {
            Ok(inner) => inner,
            Err(_) => return String::new(),
        };

        // 检查缓存
        let args_hash = compute_args_hash(args);
        if let Some(cached) = inner.cache.get(key, locale, args_hash) {
            return cached;
        }

        // 按回退链查找
        let result = inner.resolve_with_fallback(key, locale, args);

        // 缓存结果
        inner.cache.insert(key, locale, args_hash, result.clone());

        result
    }

    /// 清空缓存（语言切换时调用）
    pub fn clear_cache(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.cache.clear();
        }
    }

    /// 获取已加载的 locale 列表
    pub fn loaded_locales(&self) -> Vec<Locale> {
        if let Ok(inner) = self.inner.lock() {
            inner.ftl_contents.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// 检查 Key 是否存在于指定 locale
    pub fn has_key(&self, key: &str, locale: &Locale) -> bool {
        if let Ok(inner) = self.inner.lock() {
            inner.has_key(key, locale)
        } else {
            false
        }
    }
}

impl LocalizationInner {
    /// 按回退链查找 Key
    fn resolve_with_fallback(
        &mut self,
        key: &str,
        locale: &Locale,
        args: Option<&FluentArgs>,
    ) -> String {
        // 1. 尝试当前语言
        if let Some(text) = self.try_resolve(key, locale, args) {
            return text;
        }

        // 2. 尝试默认语言
        if *locale != self.default_locale {
            if let Some(text) = self.try_resolve(key, &self.default_locale, args) {
                return text;
            }
        }

        // 3. 尝试英语
        if *locale != Locale::EnUs && self.default_locale != Locale::EnUs {
            if let Some(text) = self.try_resolve(key, &Locale::EnUs, args) {
                return text;
            }
        }

        // 4. 返回 Key 本身（Debug 模式）或空字符串（Release 模式）
        if cfg!(debug_assertions) {
            format!("[MISSING: {}]", key)
        } else {
            String::new()
        }
    }

    /// 尝试在指定 locale 中解析 Key
    fn try_resolve(
        &self,
        key: &str,
        locale: &Locale,
        args: Option<&FluentArgs>,
    ) -> Option<String> {
        let contents = self.ftl_contents.get(locale)?;

        // 合并所有 FTL 内容并解析
        let combined = contents.join("\n");
        let resource = FluentResource::try_new(combined).ok()?;

        let mut bundle = FluentBundle::new(vec![]);
        bundle.add_resource(resource).ok()?;

        let message = bundle.get_message(key)?;
        let pattern = message.value()?;

        let mut errors = Vec::new();
        let value = bundle.format_pattern(pattern, args, &mut errors);

        if errors.is_empty() {
            return Some(value.to_string());
        }

        Some(value.to_string())
    }

    /// 检查 Key 是否存在
    fn has_key(&self, key: &str, locale: &Locale) -> bool {
        let contents = match self.ftl_contents.get(locale) {
            Some(c) => c,
            None => return false,
        };

        let combined = contents.join("\n");
        if let Ok(resource) = FluentResource::try_new(combined) {
            let mut bundle = FluentBundle::new(vec![]);
            if bundle.add_resource(resource).is_ok() {
                return bundle.get_message(key).is_some();
            }
        }

        false
    }
}

/// 计算 FluentArgs 的哈希值（用于缓存）
fn compute_args_hash(args: Option<&FluentArgs>) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    if let Some(args) = args {
        for (key, value) in args.iter() {
            key.hash(&mut hasher);
            match value {
                fluent::FluentValue::String(s) => s.hash(&mut hasher),
                fluent::FluentValue::Number(n) => {
                    format!("{:?}", n).hash(&mut hasher);
                }
                fluent::FluentValue::Custom(_) | _ => {
                    0u64.hash(&mut hasher);
                }
            }
        }
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service_with_test_ftl() -> LocalizationService {
        let service = LocalizationService::new(Locale::ZhCn);

        let zh_ftl = r#"
test-hello = 你好世界
test-greeting = { $name } 你好
test-missing = 此 Key 在英文中不存在
"#;
        let en_ftl = r#"
test-hello = Hello World
test-greeting = Hello { $name }
"#;

        service.load_ftl(Locale::ZhCn, zh_ftl).unwrap();
        service.load_ftl(Locale::EnUs, en_ftl).unwrap();

        service
    }

    #[test]
    fn resolve_basic_key() {
        let service = make_service_with_test_ftl();
        let result = service.resolve("test-hello", &Locale::ZhCn, None);
        assert_eq!(result, "你好世界");
    }

    #[test]
    fn resolve_with_args() {
        let service = make_service_with_test_ftl();
        let mut args = FluentArgs::new();
        args.set("name", "玩家");
        let result = service.resolve("test-greeting", &Locale::ZhCn, Some(&args));
        // Fluent 会用 Unicode 隔离字符包裹参数（\u{2068} 和 \u{2069}）
        assert!(result.contains("玩家"));
        assert!(result.contains("你好"));
    }

    #[test]
    fn resolve_fallback_to_default() {
        let service = make_service_with_test_ftl();
        // test-missing 在英文中不存在，应回退到中文
        let result = service.resolve("test-missing", &Locale::EnUs, None);
        assert_eq!(result, "此 Key 在英文中不存在");
    }

    #[test]
    fn resolve_missing_key_returns_placeholder() {
        let service = make_service_with_test_ftl();
        let result = service.resolve("nonexistent.key", &Locale::ZhCn, None);
        assert!(result.contains("MISSING") || result.is_empty());
    }

    #[test]
    fn load_ftl_error() {
        let service = LocalizationService::new(Locale::ZhCn);
        // 无效的 FTL 内容
        let result = service.load_ftl(Locale::ZhCn, "invalid ftl {{{{ content");
        assert!(result.is_err());
    }

    #[test]
    fn clear_cache() {
        let service = make_service_with_test_ftl();
        let _ = service.resolve("test-hello", &Locale::ZhCn, None);
        service.clear_cache();
        let result = service.resolve("test-hello", &Locale::ZhCn, None);
        assert_eq!(result, "你好世界");
    }

    #[test]
    fn has_key() {
        let service = make_service_with_test_ftl();
        assert!(service.has_key("test-hello", &Locale::ZhCn));
        assert!(!service.has_key("nonexistent", &Locale::ZhCn));
    }

    #[test]
    fn loaded_locales() {
        let service = make_service_with_test_ftl();
        let locales = service.loaded_locales();
        assert!(locales.contains(&Locale::ZhCn));
        assert!(locales.contains(&Locale::EnUs));
    }
}
