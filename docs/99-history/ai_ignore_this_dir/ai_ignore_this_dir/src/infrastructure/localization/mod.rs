//! 本地化模块：基于 Fluent 的多语言支持
//!
//! 自封装 fluent-rs + intl-memoizer，不依赖第三方 bevy_fluent 插件。
//! 参见 `docs/01-architecture/i18n-design.md` 和 `docs/08-decisions/ADR-017-国际化架构决策.md`。

mod cache;
mod component;
mod font_fallback;
mod ftl_loader;
mod locale;
mod plugin;
mod service;
mod systems;

pub use cache::LocalizedTextCache;
pub use component::LocalizedText;
pub use locale::{CurrentLocale, Locale};
pub use plugin::LocalizationPlugin;
pub use service::LocalizationService;

/// 语言切换事件（Bevy 0.18 使用 Message 机制）
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct LanguageChangedMessage {
    pub new_locale: Locale,
    pub old_locale: Locale,
}
