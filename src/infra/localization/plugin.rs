//! LocalizationPlugin — 本地化基础设施 Plugin
//!
//! 注册 LocalizationDatabase Resource、加载系统、校验系统、
//! 渲染系统、运行时审计和热重载（debug 构建）。
//!
//! 详见 `docs/03-technical/localization-design.md` §9.4

use bevy::prelude::*;

use super::audit::{AuditTimer, audit_system};
use super::cache::{LocalizedTextCache, detect_locale_change_and_clear_cache};
use super::components::render_localized_text;
use super::database::LocalizationDatabase;
use super::error::LocaleId;
use super::loader::load_all_ftl_system;
use super::validator::validation_system;

/// Locale 配置
#[derive(Resource, Clone)]
pub struct LocaleConfig {
    /// 默认语言
    pub default_locale: LocaleId,
    /// 严格模式：缺失 key 时 panic 而非返回兜底 key
    pub strict_mode: bool,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            default_locale: "en-US".into(),
            strict_mode: false,
        }
    }
}

/// Localization Plugin 配置结构
pub struct LocalizationPlugin {
    config: LocaleConfig,
}

impl Default for LocalizationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizationPlugin {
    /// 创建带有默认配置的 LocalizationPlugin
    ///
    /// `fake-locale` feature 启用时自动使用 zz-ZZ 作为默认 locale。
    pub fn new() -> Self {
        #[cfg(feature = "fake-locale")]
        let default_locale = "zz-ZZ".into();

        #[cfg(not(feature = "fake-locale"))]
        let default_locale = "en-US".into();

        Self {
            config: LocaleConfig {
                default_locale,
                strict_mode: false,
            },
        }
    }

    /// 自定义配置创建
    pub fn with_config(config: LocaleConfig) -> Self {
        Self { config }
    }
}

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── 初始化资源 ──
            .init_resource::<LocalizationDatabase>()
            .init_resource::<LocalizedTextCache>()
            .init_resource::<AuditTimer>()
            .insert_resource(self.config.clone())
            // ── 加载系统 — PreStartup（在 Startup 之前运行） ──
            .add_systems(PreStartup, load_all_ftl_system)
            // ── 校验系统 — Startup（加载完成后） ──
            .add_systems(Startup, validation_system.after(load_all_ftl_system))
            // ── 缓存清理系统 — Update（检测 locale 变化） ──
            .add_systems(Update, detect_locale_change_and_clear_cache)
            // ── UI 渲染系统 — PreUpdate（在 UI 更新之前） ──
            .add_systems(PreUpdate, render_localized_text)
            // ── 运行时审计 — Update ──
            .add_systems(Update, audit_system);

        // 设置默认 locale（如果与 en-US 不同）
        let default_locale = self.config.default_locale.clone();
        if default_locale != "en-US" {
            app.add_systems(Startup, apply_default_locale.after(validation_system));
        }

        // 热重载（仅 debug 构建，非 wasm 平台）
        #[cfg(debug_assertions)]
        #[cfg(not(target_arch = "wasm32"))]
        {
            let watcher = super::loader::create_locale_watcher();
            app.insert_non_send(watcher);
            app.add_systems(Update, super::loader::hot_reload_system);
        }
    }
}

/// Startup System: 应用默认 locale
fn apply_default_locale(mut db: ResMut<LocalizationDatabase>, config: Res<LocaleConfig>) {
    let locale = config.default_locale.clone();
    if locale != *db.current_locale() {
        db.set_locale(locale);
    }
}
