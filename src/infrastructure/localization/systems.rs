//! 文本解析系统

use bevy::prelude::*;

use super::component::LocalizedText;
use super::locale::CurrentLocale;
use super::service::LocalizationService;

/// 语言切换时重新解析所有 LocalizedText 组件
/// 仅在组件刚添加或变化时执行，平时静默
pub fn resolve_localized_texts(
    locale: Res<CurrentLocale>,
    localization: Res<LocalizationService>,
    mut query: Query<(&LocalizedText, &mut Text), Or<(Added<LocalizedText>, Changed<LocalizedText>)>>,
) {
    for (localized, mut text) in query.iter_mut() {
        let new_string = resolve_with_args(&localization, &localized.key, &locale.0, &localized.args_keys);

        // 防御性赋值：仅在文本真正改变时更新
        if text.as_str() != new_string {
            *text = Text::new(new_string);
        }
    }
}

/// 解析带参数的 Key
fn resolve_with_args(
    localization: &LocalizationService,
    key: &str,
    locale: &super::locale::Locale,
    args_keys: &Option<Vec<(String, String)>>,
) -> String {
    if let Some(keys) = args_keys {
        // 构建参数并解析
        let mut args = std::collections::HashMap::new();
        for (k, v) in keys {
            args.insert(k.clone(), v.clone());
        }
        // 简化实现：直接使用 resolve 无参数版本
        // 后续可扩展为支持参数的解析
        localization.resolve(key, locale, None)
    } else {
        localization.resolve(key, locale, None)
    }
}

/// 语言切换时清空缓存
pub fn on_language_changed(
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    localization.clear_cache();
    debug!(
        target: "localization",
        event = "cache_cleared",
        locale = %locale.0,
        "本地化缓存已清空"
    );
}

/// 启动时初始化本地化系统（加载 FTL 文件）
pub fn initialize_localization(
    localization: Res<LocalizationService>,
    locale: Res<CurrentLocale>,
) {
    use super::ftl_loader::load_locale_ftl_files;

    let locale_str = locale.0.as_str();
    let locale_dir = format!("assets/localization/{}", locale_str);

    match load_locale_ftl_files(&locale_dir) {
        Ok(content) => {
            if let Err(e) = localization.load_ftl(locale.0, &content) {
                error!(
                    target: "localization",
                    event = "ftl_load_error",
                    locale = %locale_str,
                    error = %e,
                    "FTL 文件加载失败"
                );
            } else {
                info!(
                    target: "localization",
                    event = "localization_initialized",
                    locale = %locale_str,
                    "本地化系统已初始化"
                );
            }
        }
        Err(e) => {
            // FTL 文件不存在时静默跳过（可能是首次启动，还没有翻译文件）
            debug!(
                target: "localization",
                event = "ftl_not_found",
                locale = %locale_str,
                error = %e,
                "FTL 文件未找到，使用空翻译表"
            );
        }
    }
}
